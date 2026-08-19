# sone for Dart

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

```dart
import 'package:sone/sone.dart' as s;

s.Font.load('Inter', 'fonts/Inter-Regular.ttf');

final root = s.Column(
  gap: 20,
  padding: 20,
  width: 420,
  height: 300,
  bg: 'khaki',
  cornerRadius: 28,
  children: [
    s.Column(flex: 1, cornerRadius: 20, cornerSmoothing: 0.7, bg: 'white'),
    s.Row(gap: 10, children: [
      s.Column(bg: 'lightgreen', size: 50, cornerRadius: 14),
      s.Column(bg: 'salmon', height: 50, cornerRadius: 14, flex: 1),
      s.Column(bg: 'orange', size: 50, cornerRadius: 14),
    ]),
  ],
);

s.render(root, density: 2).save('card.png');
```

The import prefix is the whole answer to the Flutter name collision: `Column`,
`Row`, `Text` and `Table` are all widgets there too.

## The shape of the API

**Every property is a named argument.** There are ~75 of them on every container
constructor, generated from the same table the setters come from
(`tool/generate_nodes.py`), so the two forms cannot drift. Run it after changing
a property:

```bash
python3 tool/generate_nodes.py && dart format lib/src/nodes.dart
```

**Cascades are the second shape**, and not a fallback — `..` evaluates to the
receiver, so setters return `void` and none of the self-type machinery the JVM
and .NET bindings need exists here. Named arguments call straight into those
setters, and the two mix freely:

```dart
s.Column(gap: 20)
  ..padding(10, 20)
  ..bg('khaki');
```

Two things only cascades can say:

- **The order filters apply in.** Named arguments apply them in declaration
  order; `..grayscale(0.5)..blur(4)` is how you choose.
- **An explicit null.** A null named argument means "unset"; the explicit null
  that clears a decoration colour to the text colour is `..underlineColor()`.

**Children go in `children:`,** where `collection-for` and `collection-if` are
the best answer to generated content in any of these languages:

```dart
s.Table(spacing: 8, children: [
  for (final record in records)
    s.TableRow(children: [
      for (final cell in record.cells) s.TableCell(children: [s.Text(cell)]),
    ]),
  if (records.isEmpty) s.TableRow(children: [s.TableCell()]),
]);
```

**Keywords are enums**, so a call site reads as `s.JustifyContent.spaceBetween`
with nothing to misspell.

**Lengths are `Object`.** Dart has no union types, so `width` takes a number or
a string and checks at the call site — `width: 100`, `width: '50%'`,
`width: 'auto'`. A bad value throws where you wrote it, not at render time.

**Text content is a String or a list:**

```dart
s.Text('Hello', size: 28)

s.Text(
  ['Hello ', s.Span('world', weight: 'bold', color: 'salmon')],
  font: 'Inter',
  size: 28,
  align: s.TextAlign.justify,
)
```

## Three names that had to move

| TypeScript | Dart | Why |
|---|---|---|
| `List()` | `Bullets` | `List<T>` is `dart:core` |
| `Path()` | `SvgPath` | `Path` is `dart:ui` in Flutter |
| `Text.size()` | `Text.size()` | kept, but see below |

`Text` mixes all three property sets, and Dart mixins linearize — two mixins
cannot declare `size` with different signatures. The font size lives on
`SpanStyleProps.fontSize`, and `Text` overrides `size` to mean it, so
`s.Text('Hi', size: 28)` is the font size the way the TypeScript API intends.
Use `width` and `height` for the box. That override is load-bearing: without it
the `size:` argument would silently set the box instead.

## Engine and output

```dart
final engine = s.Engine('assets');
engine.registerFont('Inter', File('Inter-Regular.ttf').readAsBytesSync());
engine.registerImage('logo', pngBytes);           // reachable as asset:logo

final rendering = s.render(root,
    engine: engine,
    width: 816,
    pageHeight: 1056,
    header: s.Text('Page {pageNumber} of {totalPages}'));

rendering.pdf();                                   // selectable text, one page per break
rendering.pages();                                 // one raster image per page
rendering.save('report.pdf');
rendering.savePages('page.png');                   // page-1.png, page-2.png, ...
rendering.layout();                                // the computed layout tree
rendering.metadata(s.Granularity.word);
rendering.toJsonString(pretty: true);              // the IR itself
engine.close();
```

Skia carries no system fonts, so at least one family must be registered before
any text renders. Header and footer text uses the literal tokens `{pageNumber}`
and `{totalPages}`; the engine substitutes them.

**FFI calls block the isolate.** A Flutter app should render inside
`Isolate.run` — with its own `Engine`, not a shared one, because Skia's font
collection is shared inside an engine.

## Flutter and Android

This package is pure Dart and works anywhere the Dart VM does. For a Flutter app
use [`sone_flutter`](../flutter), which carries the Android native libraries and
adds `pngAsync` / `pdfAsync` so a render does not block the UI isolate.

The loader here already knows about Android: the library lives inside the APK,
unpacked into the app's `nativeLibraryDir`, so the bare name is the only
candidate it tries.

## Installing

`dart:ffi` over the C ABI in `include/sone.h`. **The native library is not in
the package yet** — build it from a checkout:

```bash
cargo build --release -p sone-ffi
```

The binding finds it by walking up to the checkout root. Anywhere else, set
`SONE_NATIVE_LIBRARY` to the file or the directory holding it. Native assets
(`hook/build.dart`) are the eventual answer, so consumers need no toolchain.

## Development

```bash
cd bindings/dart
dart pub get
dart test
dart run example/main.dart      # renders card.png
```

33 tests, including the parity gate every sone binding owes: the same document
rendered through this binding and through `sone-cli` must come out byte for byte
identical.
