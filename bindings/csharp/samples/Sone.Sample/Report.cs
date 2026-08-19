using static Sone.Dsl;

namespace Sone.Sample;

/// <summary>
/// A paginated A4 report: running header and footer with page tokens, a table,
/// a list, an image, and an explicit page break.
/// </summary>
internal static class Report
{
    private const string Ink = "#14171a";
    private const string Muted = "#66707c";
    private const string Rule = "#e3e8ef";
    private const string Accent = "#b03a2e";

    // A4 at 96dpi is 794 x 1123. `config.width` is the width content is laid
    // out at, and the canvas grows by the margins on top of it — so the root
    // carries the content width and the config carries only the page height.
    private const double Margins = 64;
    private const double ContentWidth = 794 - (2 * Margins);
    private const double PageHeight = 1123;

    internal static Rendering Build(Engine engine) => Column(
            Title("The sone engine"),
            Text("A declarative canvas layout engine with rich international text")
                .Font("Google Sans").Size(13).Color(Muted).LineHeight(1.5),
            Divider(),

            Heading("What it does"),
            Body("A document is a tree of boxes laid out with flexbox, drawn by Skia, and "
                 + "written out as a raster image, a PDF, or SVG. The same tree paginates: "
                 + "give it a page height and the engine finds break points that do not cut "
                 + "a line of text in half.")
                .Align(TextAlign.Justify),
            Body("Every language binding is thin. The fluent builder is reimplemented per "
                 + "language and produces the same JSON IR document; the native layer is "
                 + "document in, bytes out. Layout, text and drawing exist exactly once.")
                .Align(TextAlign.Justify),

            Heading("The crates"),
            CrateTable(),

            Heading("Output formats"),
            List(
                Item("PNG, JPEG and WebP, at any density"),
                Item("Raw RGBA pixels, for handing straight to another pipeline"),
                Item("PDF, with selectable text and one page per break"),
                Item("SVG, for anything that wants vectors back out")
            ).ListStyle("disc").MarkerGap(10).Gap(7),

            Heading("How a document becomes pixels"),
            List(
                Item("The builder produces a JSON IR document — pure C#, no native code."),
                Item("The engine parses it, resolves CSS values, and compiles a node tree."),
                Item("taffy lays the tree out; text nodes measure through the shaping engine."),
                Item("Pagination walks the laid-out tree and picks break points."),
                Item("Skia paints each page, and an encoder turns it into bytes.")
            ).ListStyle("decimal").MarkerGap(10).Gap(7),

            // Everything after this starts a new page.
            PageBreak(),

            Heading("Text is the hard part"),
            Body("Skia carries no system fonts, so a family must be registered before any "
                 + "text renders — which is a feature: output does not change because a "
                 + "machine happens to have a font installed. Shaping runs through HarfBuzz, "
                 + "so complex scripts, bidirectional runs and ligatures all work.")
                .Align(TextAlign.Justify),

            Photo("fixtures/image/kouprey.jpg")
                .Width(Dim.Percent(100)).Height(240)
                .ScaleType(ScaleType.Cover)
                .Rounded(10),
            Text("The kouprey, rendered through ",
                    Code("Photo(\"…\").ScaleType(ScaleType.Cover)"))
                .Font("Google Sans").Size(9).Color(Muted),

            Heading("This document"),
            Body("Built with the C# binding. Every property below is one method call:"),
            CodeBlock(
                "root.Width(666).Render(",
                "        engine,",
                "        pageHeight: 1123,",
                "        margin: new Margin(56, 64, 56, 64),",
                "        header: RunningHeader(),",
                "        footer: RunningFooter())",
                "    .Save(\"report.pdf\");")
        )
        .Gap(18)
        .Width(ContentWidth)
        .Render(
            engine,
            pageHeight: PageHeight,
            background: "#ffffff",
            margin: new Margin(Top: 56, Right: Margins, Bottom: 56, Left: Margins),
            lastPageHeight: LastPageHeight.Uniform,
            header: RunningHeader(),
            footer: RunningFooter());

    // ── bands ────────────────────────────────────────────────────────────────

    // The engine substitutes {pageNumber} and {totalPages} during pagination —
    // a binding passes the tokens through untouched.
    private static INode RunningHeader() => Row(
            Small("sone · the Rust port"),
            Small("bindings/csharp")
        )
        .JustifyContent(JustifyContent.SpaceBetween)
        .Padding(0, Margins, 10, Margins)
        .BorderWidth(0, 0, 1, 0)
        .BorderColor(Rule);

    private static INode RunningFooter() => Row(
            Small("Apache-2.0"),
            Small("Page {pageNumber} of {totalPages}")
        )
        .JustifyContent(JustifyContent.SpaceBetween)
        .Padding(10, Margins, 0, Margins)
        .BorderWidth(1, 0, 0, 0)
        .BorderColor(Rule);

    // ── blocks ───────────────────────────────────────────────────────────────

    // Cells are left content-sized. Forcing a width on them to stretch the table
    // to the full measure derails the row layout — see the table cell
    // cross-sizing entry in docs/roadmap.md.
    private static INode CrateTable() => Table(
            HeaderRow("Crate", "Role", "Links Skia"),
            BodyRow("sone-core", "IR, CSS parsing, layout, text engine, pagination", "no"),
            BodyRow("sone-skia", "painter, shaping, image decode, encoders", "yes"),
            BodyRow("sone-ffi", "the C ABI every FFI binding speaks", "yes"),
            BodyRow("sone-cli", "render · dump-layout · dump-metadata", "yes")
        );

    private static INode HeaderRow(params string[] cells) => TableRow(
        [.. cells.Select(cell => Cell(
                Text(cell).Font("Google Sans").Size(9.5).Weight(FontWeight.Bold).Color(Ink))
            .Bg("#f4f6f9")
            .BorderColor("#cfd6e0"))]);

    private static INode BodyRow(string name, string role, string skia) => TableRow(
        Cell(Mono(name).Color(Accent)),
        Cell(Body(role).Size(9.5)),
        Cell(Body(skia).Size(9.5).Color(Muted)));

    private static TableCellNode Cell(INode content) =>
        TableCell(content)
            .Padding(9, 10)
            .BorderWidth(0, 0, 1, 0)
            .BorderColor(Rule);

    private static INode CodeBlock(params string[] lines) => Column(
            [.. lines.Select(line => Mono(line).Size(9).Color("#2c3e50"))])
        .Gap(3)
        .Padding(14)
        .Bg("#f6f8fa")
        .Rounded(8)
        .BorderWidth(1)
        .BorderColor(Rule);

    private static INode Divider() => Column().Height(1).Bg(Rule);

    private static ListItemNode Item(string text) => ListItem(Body(text));

    // ── text roles ───────────────────────────────────────────────────────────

    private static TextNode Title(string text) =>
        Text(text).Font("Google Sans").Size(30).Weight(FontWeight.Bold).Color(Ink).LineHeight(1.2);

    private static TextNode Heading(string text) =>
        Text(text).Font("Google Sans").Size(15).Weight(FontWeight.Bold).Color(Ink).LineHeight(1.35);

    private static TextNode Body(params Inline[] content) =>
        Text(content).Font("Google Sans").Size(10.5).Color(Ink).LineHeight(1.65);

    private static TextNode Small(string text) =>
        Text(text).Font("Google Sans").Size(8.5).Color(Muted);

    private static TextNode Mono(string text) =>
        Text(text).Font("Geist Mono").Size(9.5).Color(Ink);

    /// <summary>The same, as a run inside a paragraph rather than a paragraph.</summary>
    private static SpanNode Code(string text) =>
        Span(text).Font("Geist Mono").Color(Accent);
}
