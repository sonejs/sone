using System.Diagnostics;
using System.Text.Json.Nodes;
using Xunit;
using static Sone.Dsl;

namespace Sone.Tests;

/// <summary>
/// Everything that crosses the C ABI. These need a <c>cargo build -p sone-ffi</c>;
/// without one they return rather than fail, the same way the Rust C-ABI test
/// skips when there is no compiler.
/// </summary>
public class EngineTests
{
    private const string Family = "Geist Mono";

    private static string FontPath => Repo.Fixture("fixtures/font/GeistMono-Regular.ttf");

    private static bool Skip()
    {
        if (Repo.HasNative)
        {
            return false;
        }
        Console.Error.WriteLine("no native library; run `cargo build -p sone-ffi` first");
        return true;
    }

    private static Engine NewEngine()
    {
        var engine = new Engine(Repo.Root);
        engine.RegisterFontFile(Family, FontPath);
        return engine;
    }

    [Fact]
    public void RendersAPng()
    {
        if (Skip()) return;
        using var engine = NewEngine();

        var png = Column().Size(16).Bg("red").Render(engine).Png();

        Assert.Equal(new byte[] { 0x89, (byte)'P', (byte)'N', (byte)'G' }, png.Take(4));
    }

    [Fact]
    public void DensityScalesTheRaster()
    {
        if (Skip()) return;
        using var engine = NewEngine();
        var node = Column().Size(10).Bg("red");

        // Raw is 4 bytes per pixel, so the byte count is the pixel count.
        Assert.Equal(10 * 10 * 4, node.Render(engine).Raw().Length);
        Assert.Equal(20 * 20 * 4, node.Render(engine).Raw(density: 2).Length);
    }

    [Fact]
    public void RendersEveryFormat()
    {
        if (Skip()) return;
        using var engine = NewEngine();
        var rendering = Column().Size(16).Bg("teal").Render(engine);

        Assert.NotEmpty(rendering.Jpeg(quality: 0.8));
        Assert.NotEmpty(rendering.Webp());
        Assert.StartsWith("%PDF", System.Text.Encoding.ASCII.GetString(rendering.Pdf()[..4]));
        Assert.Contains("<svg", System.Text.Encoding.UTF8.GetString(rendering.Svg()));
    }

    [Fact]
    public void OnePageIsProducedPerBreak()
    {
        if (Skip()) return;
        using var engine = NewEngine();

        var pages = Column(
                Column().Height(60).Bg("red"),
                Column().Height(60).Bg("green").PageBreak(PageBreakMode.Before),
                Column().Height(60).Bg("blue").PageBreak(PageBreakMode.Before))
            .Render(engine, width: 40, pageHeight: 200)
            .Pages();

        Assert.Equal(3, pages.Count);
        Assert.All(pages, page => Assert.NotEmpty(page));
    }

    [Fact]
    public void TheFontRegistryRoundTrips()
    {
        if (Skip()) return;
        using var engine = new Engine(Repo.Root);

        Assert.False(engine.HasFont(Family));
        engine.RegisterFontFile(Family, FontPath);
        Assert.True(engine.HasFont(Family));
        Assert.Contains(Family, engine.FontFamilies());

        engine.ResetFonts();
        Assert.False(engine.HasFont(Family));

        engine.RegisterFont(Family, File.ReadAllBytes(FontPath));
        Assert.True(engine.HasFont(Family));
    }

    [Fact]
    public void RegisteredImagesResolveAsAssets()
    {
        if (Skip()) return;
        using var engine = NewEngine();
        var png = Column().Size(8).Bg("red").Render(engine).Png();

        engine.RegisterImage("logo", png);

        Assert.NotEmpty(Photo("asset:logo").Size(8).Render(engine).Png());
    }

    [Fact]
    public void LayoutComesBackAsATree()
    {
        if (Skip()) return;
        using var engine = NewEngine();

        var layout = Column(Column().Size(20).Tag("inner")).Padding(5).Render(engine).Layout();

        Assert.Equal(30, layout!["width"]!.GetValue<double>());
        Assert.Equal("inner", layout["children"]![0]!["tag"]!.GetValue<string>());
    }

    [Fact]
    public void MetadataHonoursGranularity()
    {
        if (Skip()) return;
        using var engine = NewEngine();
        var rendering = Text("hello world").Font(Family).Size(12).Render(engine);

        Assert.IsAssignableFrom<JsonNode>(rendering.Metadata());
        Assert.IsAssignableFrom<JsonNode>(rendering.Metadata(Granularity.Word));
    }

    [Fact]
    public void ABadDocumentIsAnIrError()
    {
        if (Skip()) return;
        using var engine = NewEngine();

        var error = Assert.Throws<IrException>(() => engine.Render("""{"sone":99,"root":{"type":"column"}}"""));

        Assert.Contains("unsupported IR version", error.Message);
    }

    [Fact]
    public void AMissingFontFileIsAnAssetError()
    {
        if (Skip()) return;
        using var engine = new Engine(Repo.Root);

        Assert.Throws<AssetException>(() => engine.RegisterFontFile("Nope", "does/not/exist.ttf"));
    }

    [Fact]
    public void UsingADisposedEngineThrowsRatherThanCrashing()
    {
        if (Skip()) return;
        var engine = new Engine(Repo.Root);
        engine.Dispose();
        engine.Dispose();

        Assert.Throws<ObjectDisposedException>(() => engine.HasFont(Family));
    }

    [Fact]
    public void SaveInfersTheFormatFromTheSuffix()
    {
        if (Skip()) return;
        using var engine = NewEngine();
        var directory = Directory.CreateTempSubdirectory("sone-csharp");
        try
        {
            var path = Path.Combine(directory.FullName, "card.pdf");
            Column().Size(16).Bg("red").Render(engine).Save(path);
            Assert.StartsWith("%PDF", System.Text.Encoding.ASCII.GetString(File.ReadAllBytes(path)[..4]));
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    /// <summary>
    /// The gate every binding owes: the same document must come out of this
    /// binding byte for byte the way it comes out of <c>sone-cli</c>. It catches
    /// a whole class of marshalling bugs for the price of one process spawn.
    /// </summary>
    [Fact]
    public void MatchesTheCliByteForByte()
    {
        if (Skip()) return;

        var root = Column(
            Text("Hello ", Span("world").Weight(FontWeight.Bold).Color("#c0392b"))
                .Font(Family).Size(24).LineHeight(1.4),
            Row(
                Column().Bg("lightgreen").Size(50).BorderRadius(14),
                Column().Bg("salmon").Height(50).BorderRadius(14).Flex(1)
            ).Gap(10)
        ).Gap(20).Padding(20).Size(420, 200).Bg("khaki").CornerRadius(28);

        // An absolute src, because the CLI resolves a document's assets against
        // the document's own directory and this engine resolves them against its
        // base directory — the two only agree when the path is absolute.
        var rendering = root.Render(
            engine: new Engine(Repo.Root),
            density: 2,
            fonts: [new FontSource(Family, FontPath)]);

        var directory = Directory.CreateTempSubdirectory("sone-parity");
        try
        {
            var document = Path.Combine(directory.FullName, "doc.json");
            File.WriteAllText(document, rendering.Json(indented: true));

            var fromCli = Path.Combine(directory.FullName, "cli.png");
            RunCli(document, fromCli);

            Assert.Equal(File.ReadAllBytes(fromCli), rendering.Png());
        }
        finally
        {
            directory.Delete(recursive: true);
        }
    }

    private static void RunCli(string document, string output)
    {
        var process = Process.Start(new ProcessStartInfo("cargo")
        {
            ArgumentList = { "run", "-q", "-p", "sone-cli", "--", "render", document, "--density", "2", "-o", output },
            WorkingDirectory = Repo.Root,
            RedirectStandardError = true,
        }) ?? throw new InvalidOperationException("could not start cargo");

        var stderr = process.StandardError.ReadToEnd();
        process.WaitForExit();
        Assert.True(process.ExitCode == 0, $"sone-cli failed: {stderr}");
    }
}
