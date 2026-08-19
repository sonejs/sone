# Sone for .NET

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

```csharp
using Sone;
using static Sone.Dsl;

Font.Load("Inter", "fonts/Inter-Regular.ttf");

var root = Column(
    Text("Hello").Size(28).Weight(FontWeight.Bold),
    Row(
        Column().Bg("salmon").Size(50).Rounded(14),
        Column().Bg("orange").Size(50).Rounded(14)
    ).Gap(10)
).Gap(20).Padding(20).Bg("khaki").CornerRadius(28);

root.Render(density: 2).Save("card.png");
```

`using static Sone.Dsl;` is the whole import ceremony: the factories then read
exactly as they do in the TypeScript engine that is this project's reference.

## The shape of the API

**Fluent properties are generic extension methods.** `Gap`, `Padding`, `Bg` and
the rest are declared once against a marker interface and return the caller's
own type, so `Column().Bg("salmon").Size(50)` still hands back a `ColumnNode`.
That is also what lets `TextNode` be a box, a styled run and a paragraph at
once, which single inheritance would forbid.

**Lengths are a `Dim`.** Implicit conversions mean the `number | "auto" | "%"`
union of the IR is one method, not three overloads:

```csharp
.Width(100)   .Width("50%")   .Width(Dim.Percent(50))   .Width(Dim.Auto)
```

**Keywords are structs with named constants, and still accept strings.**
`Justify.SpaceBetween` gets you completion; `"space-between"` still compiles, so
a value the engine understands is never unreachable from here.

**Shorthands use named arguments.** CSS 1–4 value semantics, with omitted sides
resolving the CSS way — right follows top, bottom follows top, left follows
right:

```csharp
.Padding(20)                 .Padding(10, 20)                .Padding(top: 8, left: 4)
```

**Generated children go in as collection expressions:**

```csharp
Table([.. rows.Select(r => TableRow([.. r.Cells.Select(c => TableCell(Text(c)))]))])
```

A null child is dropped, so `cond ? Badge() : null` needs no filtering.

## Naming

Methods are PascalCase; against the TypeScript reference that is a one-letter
difference, so examples transfer unchanged. Two names had to move:

| TypeScript | C# | Why |
|---|---|---|
| `Path()` | `SvgPath()` | `System.IO.Path` arrives with the implicit usings |
| `Column`, `Text`, … the classes | `ColumnNode`, `TextNode`, … | a static factory and a type cannot share a name and stay invocable |

`List()` and `Span()` survive: `List<T>` and `Span<T>` are distinguished by
arity, so the zero-arity factories never collide.

## Engine and output

```csharp
using var engine = new Engine(baseDir: "assets");
engine.RegisterFont("Inter", File.ReadAllBytes("Inter-Regular.ttf"));
engine.RegisterImage("logo", pngBytes);          // reachable as asset:logo

var rendering = root.Render(engine, width: 816, pageHeight: 1056,
                            header: Text("Page {pageNumber} of {totalPages}"));

byte[] pdf = rendering.Pdf();                    // selectable text, one page per break
var pages  = rendering.Pages();                  // one raster image per page
await rendering.SaveAsync("report.pdf");         // renders off-thread, writes async
JsonNode layout = rendering.Layout();            // the computed layout tree
string json = rendering.Json(indented: true);    // the IR itself
```

Skia carries no system fonts, so at least one family must be registered before
any text renders. `Font.Load` does it on the process-wide `Engine.Default` for
scripts that do not want to own an engine.

Header and footer text uses the literal tokens `{pageNumber}` and
`{totalPages}`; the engine substitutes them during pagination.

**One engine per thread.** Skia's font collection is shared inside an engine, so
every call takes a lock. Give each thread its own `Engine` for real parallelism
rather than sharing one.

Failures arrive as `IrException`, `AssetException` or `RenderException`, all
under `SoneException`, mapped from the C ABI's status codes.

## Building from source

The managed assembly talks to the native library over the C ABI in
`include/sone.h`.

```bash
cargo build --release -p sone-ffi        # produces target/release/libsone.dylib
cd bindings/csharp
dotnet build
dotnet test
```

The tests find that library themselves by walking up to the checkout root. To
point at one somewhere else, set `SONE_NATIVE_LIBRARY` to the file or to the
directory holding it. A NuGet consumer needs none of this — the library ships in
`runtimes/{rid}/native` and the default resolver finds it.

`dotnet test` includes the parity gate every sone binding owes: the same
document rendered through this binding and through `sone-cli` must come out byte
for byte identical.

## Samples

```bash
dotnet run --project samples/Sone.Sample        # writes into target/samples
```

Three PDFs, each exercising a different part of the engine:

| | |
|---|---|
| `report.pdf` | a paginated A4 report — running header and footer with `{pageNumber}` tokens, a table, ordered and unordered lists, an image, an explicit page break |
| `card.pdf` | the single-page card from the top of this file, as a PDF rather than a PNG — the tree does not change, only the method called at the end |
| `scripts.pdf` | Khmer, Arabic, Hebrew and mixed bidirectional runs, plus span decorations |

Two things the report demonstrates that are easy to get wrong:

- **`config.width` is the width content is laid out at, and the canvas grows by
  the margins on top of it.** So a 794pt A4 page with 64pt margins wants a root
  of 666, not 794 — set the page width on the root and only the page *height* in
  the config.
- **Table cells are content-sized.** Forcing a width on them to stretch a table
  to the full measure derails the row layout; see the table cell cross-sizing
  entry in [docs/roadmap.md](../../docs/roadmap.md).

## Design

Every sone binding is thin. The fluent builder is reimplemented per language and
produces the same JSON **IR document**; the native layer is document-in,
bytes-out. Layout, text and drawing exist exactly once, in Rust.

That means the interesting parts of this package are pure C#: the tree is built
and serialized with no native code involved, and `Json()` works before an engine
exists. The document is written straight to UTF-8 with `Utf8JsonWriter` and
handed to the engine as a pinned buffer, so the IR never round-trips through
UTF-16, and both `LibraryImport` and source-generated JSON keep the assembly
trimmable and NativeAOT-clean.
