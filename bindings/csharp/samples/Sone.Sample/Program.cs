using Sone;
using Sone.Sample;
using static Sone.Dsl;

// Three PDFs, each exercising a different part of the engine: a paginated
// report, a single-page card, and a multi-script page.
//
//     dotnet run --project samples/Sone.Sample -- [output-directory]

var repo = Checkout.Root;
var outputDirectory = args.Length > 0 ? args[0] : Path.Combine(repo, "target", "samples");
Directory.CreateDirectory(outputDirectory);

Checkout.UseLocalNativeLibrary();

using var engine = new Engine(repo);
engine.RegisterFontFile("Google Sans", Font("GoogleSans-VariableFont_GRAD,opsz,wght.ttf"));
engine.RegisterFontFile("Geist Mono", Font("GeistMono-Regular.ttf"));
engine.RegisterFontFile("Moul", Font("Moul-Regular.ttf"));
engine.RegisterFontFile("Noto Sans Khmer", Font("NotoSansKhmer.ttf"));
engine.RegisterFontFile("Noto Sans Arabic", Font("NotoSansArabic.ttf"));
engine.RegisterFontFile("Noto Sans Hebrew", Font("NotoSansHebrew.ttf"));

Write("report.pdf", Report.Build(engine));
Write("card.pdf", Card.Build(engine));
Write("scripts.pdf", Scripts.Build(engine));

string Font(string name) => Path.Combine(repo, "fixtures", "font", name);

void Write(string name, Rendering rendering)
{
    var path = Path.Combine(outputDirectory, name);
    rendering.Save(path);
    var bytes = new FileInfo(path).Length;
    Console.WriteLine($"{path}  {bytes / 1024.0:F1} KB  {Pdf.CountPages(path)} page(s)");
}
