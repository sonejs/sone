using static Sone.Dsl;

namespace Sone.Sample;

/// <summary>
/// The reason the engine exists: complex scripts and bidirectional text, shaped
/// through HarfBuzz and still selectable in the PDF.
/// </summary>
internal static class Scripts
{
    private const string Ink = "#14171a";
    private const string Muted = "#66707c";

    internal static Rendering Build(Engine engine) => Column(
            Text("International text").Font("Google Sans").Size(26).Weight(FontWeight.Bold).Color(Ink),
            Text("Four scripts, three writing directions, one layout pass.")
                .Font("Google Sans").Size(11).Color(Muted),
            Column().Height(1).Bg("#e3e8ef"),

            Sample(
                "Khmer",
                Text("ភាសាខ្មែរ").Font("Moul").Size(24).Color(Ink).LineHeight(1.6),
                Text("សូនេ គឺជាម៉ាស៊ីនរៀបចំប្លង់ ដែលគាំទ្រការសរសេរជាភាសាអន្តរជាតិ។")
                    .Font("Noto Sans Khmer").Size(13).Color(Ink).LineHeight(1.9)),

            Sample(
                "Arabic — right to left",
                Text("مرحبا بالعالم").Font("Noto Sans Arabic").Size(24).Color(Ink)
                    .BaseDir(BaseDir.Rtl).Align(TextAlign.Right).LineHeight(1.7),
                Text("سونه محرك تخطيط يدعم النصوص العالمية.")
                    .Font("Noto Sans Arabic").Size(13).Color(Ink)
                    .BaseDir(BaseDir.Rtl).Align(TextAlign.Right).LineHeight(1.9)),

            Sample(
                "Hebrew — right to left",
                Text("שלום עולם").Font("Noto Sans Hebrew").Size(24).Color(Ink)
                    .BaseDir(BaseDir.Rtl).Align(TextAlign.Right).LineHeight(1.6),
                Text("סונה הוא מנוע פריסה התומך בטקסט בינלאומי.")
                    .Font("Noto Sans Hebrew").Size(13).Color(Ink)
                    .BaseDir(BaseDir.Rtl).Align(TextAlign.Right).LineHeight(1.8)),

            Sample(
                "Mixed runs",
                Text("The word ",
                        Span("مرحبا").Font("Noto Sans Arabic").Color("#b03a2e"),
                        " sits inside an English sentence, and the bidi algorithm sorts it out.")
                    .Font("Google Sans").Size(13).Color(Ink).LineHeight(1.8).BaseDir(BaseDir.Auto),
                Text("Decorations travel with the run: ",
                        Span("underlined").Underline().UnderlineColor("#b03a2e"),
                        ", ",
                        Span("struck through").LineThrough(),
                        ", ",
                        Span("highlighted").Highlight("#ffe08a"),
                        ".")
                    .Font("Google Sans").Size(13).Color(Ink).LineHeight(1.8))
        )
        .Gap(20)
        .Padding(48)
        .Width(620)
        .Bg("#ffffff")
        .Render(engine);

    private static INode Sample(string label, params INode[] body) => Column(
            [Text(label).Font("Geist Mono").Size(9).Color(Muted).LetterSpacing(1.2), .. body])
        .Gap(8)
        .Padding(18)
        .Bg("#f8fafc")
        .Rounded(12)
        .BorderWidth(1)
        .BorderColor("#e3e8ef");
}
