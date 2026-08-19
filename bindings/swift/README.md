# sone for Swift

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

```swift
import Sone

try Font.load("Inter", "fonts/Inter-Regular.ttf")

let root = Column {
    Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white")

    Row {
        Column().bg("lightgreen").size(50).borderRadius(14)
        Column().bg("salmon").height(50).borderRadius(14).flex(1)
    }
    .gap(10)
}
.gap(20).padding(20).size(420, 300).bg("khaki")

try render(root, density: 2).save("card.png")
```

## The shape of the API

**A result builder for children, modifiers for properties** — the shape every
Swift reader already knows from SwiftUI. `@NodeBuilder` is what makes `if` and
`for` work inside the block:

```swift
Table {
    for record in records {
        TableRow {
            for cell in record.cells { TableCell { Text(cell) } }
        }
    }
    if records.isEmpty {
        TableRow { TableCell {} }
    }
}
```

**`Self` comes free.** The property methods live in extensions on
class-constrained protocols, so `Column().bg("salmon").size(50)` stays a
`Column` with none of the self-type machinery the JVM and .NET bindings need.

**Literal conformances collapse the length union.** `Dim` is
`ExpressibleByIntegerLiteral`, `ByFloatLiteral` and `ByStringLiteral`, so the
IR's `number | "auto" | "%"` is one method:

```swift
.width(100)     .width("50%")     .width(.auto)     .width(.percent(50))
```

Same for `Track` (`.fr(1)`, `.auto`), `Weight` (`.bold`, `700`) and `Margin`.

**Keywords are enums**, so a call site is `.spaceBetween` with no type to name.

**Paragraph content is its own builder:**

```swift
Text {
    "Hello "
    Span("world").weight(.bold).color("salmon")
}
.font("Inter").size(28).align(.justify)
```

`Text("Hello")` is there too for the common case.

## One name that had to move

`List` is SwiftUI's, so the list is **`Bullets`**. `Path` would be too, so the
path is **`SvgPath`** — consistent with the JVM and .NET bindings.

## Engine and output

```swift
let engine = Engine("assets")
try engine.registerFont("Inter", Data(contentsOf: fontURL))
try engine.registerImage("logo", pngBytes)         // reachable as asset:logo

let rendering = render(root,
                       engine: engine,
                       width: 816,
                       pageHeight: 1056,
                       header: Text("Page {pageNumber} of {totalPages}"))

try rendering.pdf()                                // selectable text, one page per break
try rendering.pages()                              // one raster image per page
try rendering.save("report.pdf")
try rendering.savePages("page.png")                // page-1.png, page-2.png, ...
try rendering.layoutJSON()                         // the computed layout tree
try rendering.metadataJSON(.word)
rendering.toJSON()                                 // the IR itself
engine.close()
```

`layoutJSON()` and `metadataJSON()` hand back the engine's JSON as a `String`,
so this package does not force a decoding strategy on anyone.

Skia carries no system fonts, so at least one family must be registered before
any text renders. Header and footer text uses the literal tokens `{pageNumber}`
and `{totalPages}`; the engine substitutes them.

**One engine per thread.** Skia's font collection is shared inside an engine, so
every call takes the lock. Give each thread its own `Engine` for real
parallelism.

Failures are a single `SoneError` with a `kind` of `.ir`, `.asset`, `.render` or
`.invalidArgument` — Swift's error handling reads better with one type and a
discriminant than with four.

## Platforms

| | |
|---|---|
| macOS 13+ | arm64 |
| iOS / iPadOS 12+ | arm64 device |
| iOS Simulator 14+ | arm64 (Apple Silicon) |

All three ship as slices of one `Sone.xcframework`. iPadOS is iOS, so an iPad
app needs nothing extra; Mac Catalyst rides on the iOS slice. tvOS, watchOS and
visionOS are not built — rust-skia publishes no prebuilt for them, and building
Skia from source for each is a much larger job.

## Installing

The C ABI in `include/sone.h`, linked **statically** from an XCFramework. Static
rather than a dylib because iOS will not load an arbitrary dynamic library from
a package, and because one linking model for every platform is simpler than two.

That choice is why the Swift layer calls the C functions directly instead of
resolving them with `dlsym`: a static archive only contributes the object files
something references, so late-binding the symbols would let the linker drop
every one of them first.

**Build the XCFramework before building the package:**

```bash
tools/build-apple.sh          # writes bindings/swift/Sone.xcframework
```

It is a ~200 MB build artifact and is not committed. The script cross-compiles
three slices in about a minute, because the feature set is chosen to match a
prebuilt Skia that rust-skia publishes — see the comment at the top of the
script before changing any feature.

One consequence of that feature set is worth knowing: the Apple slices are built
without `embed-freetype`, so Skia rasterizes glyphs through the platform font
engine rather than its bundled FreeType. On macOS this makes no difference the
parity gate can see — the same document still comes out of this package byte for
byte identical to `sone-cli`.

## Development

```bash
tools/build-apple.sh
cd bindings/swift
swift test
```

30 tests on macOS, including the parity gate every sone binding owes: the same
document rendered through this binding and through `sone-cli` must come out byte
for byte identical. That one is macOS-only — `Process` does not exist on iOS —
and skips itself elsewhere; the other 29 run anywhere.

To check the iOS slices without an Xcode project:

```bash
# Compiles the Swift layer against the iOS SDK
xcrun --sdk iphoneos swiftc -target arm64-apple-ios13.0 -typecheck \
  -I Sone.xcframework/ios-arm64/Headers Sources/Sone/*.swift

# Runs the engine inside a simulator
xcrun simctl spawn "iPhone 16" /path/to/your-simulator-binary
```
