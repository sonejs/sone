using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace Sone;

/// <summary>The granularity of the boxes <see cref="Rendering.Metadata"/> returns.</summary>
public readonly record struct Granularity(string Value)
{
    public static readonly Granularity Node = new("node");
    public static readonly Granularity Line = new("line");
    public static readonly Granularity Word = new("word");

    public static implicit operator Granularity(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>A font family the document carries with it.</summary>
/// <param name="Name">The family name text refers to.</param>
/// <param name="Src">A path, resolved against the engine's base directory.</param>
public readonly record struct FontSource(string Name, string Src);

/// <summary>Document-level render configuration.</summary>
public sealed record DocumentConfig
{
    /// <summary>
    /// Fonts the document declares, loaded at render time. An alternative to
    /// registering on the engine, and the only way to make a document
    /// self-contained enough for another sone engine — the CLI, say — to render
    /// it identically.
    /// </summary>
    public IReadOnlyList<FontSource>? Fonts { get; init; }

    /// <summary>Canvas width. Unset means the root's own width.</summary>
    public double? Width { get; init; }

    /// <summary>Canvas height. Unset means the content's height.</summary>
    public double? Height { get; init; }

    /// <summary>A CSS colour painted behind everything.</summary>
    public string? Background { get; init; }

    /// <summary>Raster scale factor. A render-time density overrides this.</summary>
    public double? Density { get; init; }

    /// <summary>Turn the document into pages of this height.</summary>
    public double? PageHeight { get; init; }

    /// <summary>Page margins, inside which the header, content and footer sit.</summary>
    public Margin? Margin { get; init; }

    /// <summary>Whether the last page is full height or shrinks to its content.</summary>
    public LastPageHeight? LastPageHeight { get; init; }

    /// <summary>
    /// Drawn at the top of every page. Use the literal tokens
    /// <c>{pageNumber}</c> and <c>{totalPages}</c> — the engine substitutes them.
    /// </summary>
    public INode? Header { get; init; }

    /// <inheritdoc cref="Header"/>
    public INode? Footer { get; init; }
}

/// <summary>
/// A node plus its render configuration, with one method per output format.
/// </summary>
public sealed class Rendering
{
    private static readonly JsonWriterOptions Compact = new()
    {
        // The engine reads UTF-8 directly, so escaping non-ASCII would only make
        // the document bigger — and unreadable for anyone debugging Khmer or
        // Arabic content.
        Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
    };

    private readonly INode _root;
    private readonly DocumentConfig _config;
    private readonly Engine? _engine;
    private byte[]? _cached;

    internal Rendering(INode root, DocumentConfig config, Engine? engine)
    {
        _root = root;
        _config = config;
        _engine = engine;
    }

    /// <summary>The engine this renders on: the one passed in, or the process-wide default.</summary>
    public Engine Engine => _engine ?? Sone.Engine.Default;

    // ── the document ─────────────────────────────────────────────────────────

    /// <summary>The IR document as JSON.</summary>
    public string Json(bool indented = false)
    {
        if (!indented)
        {
            var utf8 = Document();
            return Encoding.UTF8.GetString(utf8, 0, utf8.Length - 1);
        }
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream, new JsonWriterOptions
               {
                   Indented = true,
                   Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
               }))
        {
            Write(writer);
        }
        return Encoding.UTF8.GetString(stream.ToArray());
    }

    // ── outputs ──────────────────────────────────────────────────────────────

    public byte[] Png(double? density = null) =>
        Engine.Render(Document(), OutputFormat.Png, density, 1.0, false);

    public byte[] Jpeg(double quality = 1.0, double? density = null) =>
        Engine.Render(Document(), OutputFormat.Jpeg, density, quality, false);

    public byte[] Webp(double quality = 1.0, double? density = null) =>
        Engine.Render(Document(), OutputFormat.Webp, density, quality, false);

    /// <summary>Raw RGBA pixels, row-major, unpremultiplied.</summary>
    public byte[] Raw(double? density = null) =>
        Engine.Render(Document(), OutputFormat.Raw, density, 1.0, false);

    /// <summary>A PDF. With <c>PageHeight</c> set, one page per break and selectable text.</summary>
    public byte[] Pdf() =>
        Engine.Render(Document(), OutputFormat.Pdf, null, 1.0, false);

    public byte[] Svg() =>
        Engine.Render(Document(), OutputFormat.Svg, null, 1.0, false);

    /// <summary>One raster image per page. Requires <see cref="DocumentConfig.PageHeight"/>.</summary>
    public IReadOnlyList<byte[]> Pages(OutputFormat format = OutputFormat.Png, double? density = null, double quality = 1.0) =>
        Engine.RenderPages(Document(), format, density, quality, false);

    /// <summary>Render and write to <paramref name="path"/>, inferring the format from its suffix.</summary>
    public string Save(string path, double? density = null, double quality = 1.0)
    {
        File.WriteAllBytes(path, Encode(FormatFor(path), density, quality));
        return path;
    }

    /// <inheritdoc cref="Save"/>
    public async Task<string> SaveAsync(string path, double? density = null, double quality = 1.0, CancellationToken cancellationToken = default)
    {
        var format = FormatFor(path);
        // The engine call blocks, so it does not belong on the caller's thread.
        var bytes = await Task.Run(() => Encode(format, density, quality), cancellationToken).ConfigureAwait(false);
        await File.WriteAllBytesAsync(path, bytes, cancellationToken).ConfigureAwait(false);
        return path;
    }

    /// <summary>Write <c>name-1.png</c>, <c>name-2.png</c>, … next to <paramref name="path"/>.</summary>
    public IReadOnlyList<string> SavePages(string path, double? density = null, double quality = 1.0)
    {
        var format = FormatFor(path, fallback: OutputFormat.Png);
        var stem = Path.Combine(
            Path.GetDirectoryName(path) ?? "",
            Path.GetFileNameWithoutExtension(path));
        var suffix = Path.GetExtension(path);

        var pages = Pages(format, density, quality);
        var written = new List<string>(pages.Count);
        for (var i = 0; i < pages.Count; i++)
        {
            var name = $"{stem}-{i + 1}{suffix}";
            File.WriteAllBytes(name, pages[i]);
            written.Add(name);
        }
        return written;
    }

    // ── introspection ────────────────────────────────────────────────────────

    /// <summary>The computed layout tree.</summary>
    public JsonNode Layout() =>
        JsonNode.Parse(Engine.DumpLayout(Document())) ?? throw new SoneException("empty layout dump");

    /// <summary>Dataset-style boxes at node, line or word granularity.</summary>
    public JsonNode Metadata(Granularity granularity = default)
    {
        // `default(Granularity)` carries a null Value, which is the price of a
        // struct with a compile-time default. Node is the engine's own default.
        var value = string.IsNullOrEmpty(granularity.Value) ? Granularity.Node.Value : granularity.Value;
        return JsonNode.Parse(Engine.DumpMetadata(Document(), value))
               ?? throw new SoneException("empty metadata dump");
    }

    // ── internals ────────────────────────────────────────────────────────────

    private byte[] Encode(OutputFormat format, double? density, double quality) => format switch
    {
        OutputFormat.Png => Png(density),
        OutputFormat.Jpeg => Jpeg(quality, density),
        OutputFormat.Webp => Webp(quality, density),
        OutputFormat.Raw => Raw(density),
        OutputFormat.Pdf => Pdf(),
        OutputFormat.Svg => Svg(),
        _ => throw new ArgumentOutOfRangeException(nameof(format)),
    };

    private static OutputFormat FormatFor(string path, OutputFormat? fallback = null)
    {
        var suffix = Path.GetExtension(path).ToLowerInvariant();
        return suffix switch
        {
            ".png" => OutputFormat.Png,
            ".jpg" or ".jpeg" => OutputFormat.Jpeg,
            ".webp" => OutputFormat.Webp,
            ".pdf" => OutputFormat.Pdf,
            ".svg" => OutputFormat.Svg,
            ".raw" or ".rgba" => OutputFormat.Raw,
            _ => fallback ?? throw new ArgumentException(
                $"cannot infer an output format from \"{path}\"", nameof(path)),
        };
    }

    /// <summary>The document as NUL-terminated UTF-8, built once and reused.</summary>
    internal byte[] Document()
    {
        if (_cached is not null)
        {
            return _cached;
        }
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream, Compact))
        {
            Write(writer);
        }
        var json = stream.ToArray();
        // The C ABI takes a C string; one trailing byte saves a second copy.
        _cached = new byte[json.Length + 1];
        json.CopyTo(_cached, 0);
        return _cached;
    }

    private void Write(Utf8JsonWriter writer)
    {
        writer.WriteStartObject();
        writer.WriteNumber("sone", Ir.Version);

        if (_config.Fonts is { Count: > 0 } fonts)
        {
            writer.WritePropertyName("fonts");
            writer.WriteStartArray();
            foreach (var font in fonts)
            {
                writer.WriteStartObject();
                writer.WriteString("name", font.Name);
                writer.WriteString("src", font.Src);
                writer.WriteEndObject();
            }
            writer.WriteEndArray();
        }

        if (HasConfig)
        {
            writer.WritePropertyName("config");
            writer.WriteStartObject();
            WriteNumber(writer, "width", _config.Width);
            WriteNumber(writer, "height", _config.Height);
            if (_config.Background is not null)
            {
                writer.WriteString("background", _config.Background);
            }
            WriteNumber(writer, "density", _config.Density);
            WriteNumber(writer, "pageHeight", _config.PageHeight);
            if (_config.Margin is { } margin)
            {
                writer.WritePropertyName("margin");
                Ir.WriteValue(writer, margin.ToIr());
            }
            if (_config.LastPageHeight is { } last)
            {
                writer.WriteString("lastPageHeight", last.Value);
            }
            if (_config.Header is not null)
            {
                writer.WritePropertyName("header");
                Ir.WriteNode(writer, _config.Header);
            }
            if (_config.Footer is not null)
            {
                writer.WritePropertyName("footer");
                Ir.WriteNode(writer, _config.Footer);
            }
            writer.WriteEndObject();
        }

        writer.WritePropertyName("root");
        Ir.WriteNode(writer, _root);
        writer.WriteEndObject();
    }

    private bool HasConfig =>
        _config.Width is not null || _config.Height is not null || _config.Background is not null
        || _config.Density is not null || _config.PageHeight is not null || _config.Margin is not null
        || _config.LastPageHeight is not null || _config.Header is not null || _config.Footer is not null;

    private static void WriteNumber(Utf8JsonWriter writer, string name, double? value)
    {
        if (value is { } number)
        {
            writer.WriteNumber(name, number);
        }
    }
}

/// <summary>Turns a node into something renderable.</summary>
public static class RenderExtensions
{
    /// <summary>
    /// Wrap a node with render configuration. Call a format method on the result
    /// to get bytes:
    /// <code>root.Render(width: 420, density: 2).Save("card.png");</code>
    /// </summary>
    public static Rendering Render(
        this INode root,
        Engine? engine = null,
        double? width = null,
        double? height = null,
        string? background = null,
        double? density = null,
        double? pageHeight = null,
        Margin? margin = null,
        LastPageHeight? lastPageHeight = null,
        INode? header = null,
        INode? footer = null,
        IReadOnlyList<FontSource>? fonts = null)
    {
        ArgumentNullException.ThrowIfNull(root);
        return new Rendering(root, new DocumentConfig
        {
            Fonts = fonts,
            Width = width,
            Height = height,
            Background = background,
            Density = density,
            PageHeight = pageHeight,
            Margin = margin,
            LastPageHeight = lastPageHeight,
            Header = header,
            Footer = footer,
        }, engine);
    }

    /// <inheritdoc cref="Render(INode,Engine?,double?,double?,string?,double?,double?,Margin?,LastPageHeight?,INode?,INode?,System.Collections.Generic.IReadOnlyList{FontSource})"/>
    public static Rendering Render(this INode root, DocumentConfig config, Engine? engine = null)
    {
        ArgumentNullException.ThrowIfNull(root);
        ArgumentNullException.ThrowIfNull(config);
        return new Rendering(root, config, engine);
    }
}
