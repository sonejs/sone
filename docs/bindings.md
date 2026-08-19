# Bindings

How a language binding is put together, and the surface design for each one.

---

## The shape every binding takes

Two layers, always:

1. **A fluent builder in the host language** that produces an IR document. No
   FFI, no native types — just building a JSON structure. This is where each
   language gets to be idiomatic.
2. **A thin native layer**: document in, bytes out. Font registration, asset
   registration, render, `dump_layout`, `dump_metadata`.

The Python binding is the reference implementation of this split:
`bindings/python/python/sone/_nodes.py` is 450 lines of pure Python builders,
and `bindings/python/src/lib.rs` is 227 lines that only ever sees a JSON string.

Why this way rather than exposing the node tree over FFI:

- The builder API is ~140 tiny methods. Writing them in the host language is far
  less code than marshalling each one across a boundary, and infinitely easier
  to make idiomatic.
- One IR document is one FFI call, so per-property overhead disappears.
- A binding can be written and tested with no native code at all, then wired up.
- The IR is versioned (`"sone": 1`), so a binding pins a contract, not an ABI.

### Non-negotiables for a new binding

- **`core.ts` method for method.** Names may be adapted to the host convention,
  but with an alias back to the TypeScript spelling so examples transfer. The
  *shape* is the binding's to choose — Ruby nests blocks and Kotlin uses a DSL
  receiver, because fluent chaining reads worse in both.
- **`Engine` owning fonts and the asset cache**, plus a process-wide default so
  simple scripts do not have to create one.
- **One engine per thread.** Skia's font collection is shared inside an engine;
  do not offer concurrent rendering on a single handle. Python holds a mutex and
  releases the GIL around it.
- **An error type per failure class** — IR, asset, render — mapped from
  `SoneError::exit_code()`.
- **A parity test** asserting the binding produces bytes identical to
  `sone-cli` for the same document. Cheap, and it catches an entire class of
  marshalling bugs.

### Two things that always bite

- `Photo(bytes)` must become a `data:` URL; only `str` sources pass through.
- Headers and footers use literal `{pageNumber}` / `{totalPages}` tokens,
  substituted by `pagination.rs`. A binding may offer a callback that renders to
  those tokens, but the engine only sees the tokens.

---

## Rust — the `sone` crate

Not a binding: the engine's own crate, and the only one with no FFI in the path.
The builder produces `sone_core::ir::Node` directly, so there is no IR string to
serialize and no boundary to cross.

```rust
use sone::prelude::*;

let root = column()
    .gap(20).padding(20).size(420, 300).bg("khaki").corner_radius(28)
    .child(column().flex(1).corner_radius(20).corner_smoothing(0.7).bg("white"))
    .child(row().gap(10)
        .child(column().bg("lightgreen").square(50).corner_radius(14))
        .child(column().bg("salmon").height(50).corner_radius(14).flex(1)));

render(root).density(2).save("card.png")?;
```

Rust settles two of the recurring decisions for free and one differently:

- **The self type** is just `Self` on an owned builder — no recursive generics,
  no extension methods, no cascades.
- **`Text::size` versus `Column::size`** is not a collision at all. One struct
  per node type means they are simply different methods, which is the first time
  in this list that rule has not needed a hand-written resolution.
- **The length union goes away.** `Dim` is already an enum, so `.width(100)`
  takes a number through `From` and the other two cases are `Dim::Percent` and
  `Dim::Auto`. No string parsing: `"50%"` is a JavaScript affordance.

The shared properties are macro-generated per type — `impl_layout_props!(Column,
Row, …)` — which is the same answer Ruby's class macros and C#'s generic
extensions gave, for the same reason: no inheritance.

Numbers go through a `Num` trait so `.gap(20)` compiles next to `.gap(20.0)`,
and `render` is a default feature so a program that only produces IR can drop
Skia entirely.

One thing the crate name cost: `sone-ffi` also builds a lib called `sone`, and
both producing an rlib collided in the target directory. `sone-ffi` no longer
builds one — nothing depended on it from Rust.

`cargo test -p sone` runs 35 tests including the parity gate.

---

## Python — shipped

PyO3 + maturin, `abi3-py39`, so one wheel covers CPython 3.9 and up.

```python
from sone import Column, Row, Text, Font, sone

Font.load("Inter", "fonts/Inter-Regular.ttf")

root = (
    Column(
        Text("Hello").size(28).weight("bold"),
        Row(
            Column().bg("salmon").size(50).rounded(14),
            Column().bg("orange").size(50).rounded(14),
        ).gap(10),
    )
    .gap(20).padding(20).bg("khaki").cornerRadius(28)
)

sone(root).save("card.png", density=2)
```

camelCase primary with snake_case aliases generated at class-creation time.
`Text.size()` is the font size, not the box size, matching TypeScript's
`Omit<LayoutPropsBuilder, "size">`. Fluent chaining is fully typed via a
`TypeVar` bound to `Node`, so every method infers its own class back.

Separate cargo workspace — see [porting notes](porting-notes.md) for why.

---

## C# — working, not yet packaged

`LibraryImport` over `include/sone.h`, targeting `net8.0`, published as **`Sone`**
on NuGet. There is no PyO3 equivalent for .NET, so the C ABI is the whole
contract — which is why it grew `sone_register_font_file`, `sone_has_font`,
`sone_font_families`, `sone_reset_fonts`, `sone_render_pages`,
`sone_dump_layout` and `sone_dump_metadata`. Every remaining FFI binding needs
those too.

```csharp
using Sone;
using static Sone.Dsl;

Font.Load("Inter", "fonts/Inter-Regular.ttf");

var root = Column(
    Column().Flex(1).CornerRadius(20).CornerSmoothing(0.7).Bg("white"),
    Row(
        Column().Bg("lightgreen").Size(50).BorderRadius(14),
        Column().Bg("salmon").Height(50).BorderRadius(14).Flex(1)
    ).Gap(10)
).Gap(20).Padding(20).Size(420, 300).Bg("khaki");

root.Render(density: 2).Save("card.png");
```

C# forces four decisions, and answers three of them better than Java did:

- **The self type.** Fluent properties are generic extension methods over marker
  interfaces — `static T Gap<T>(this T n, double v) where T : ILayoutNode` — so
  `T` infers to `ColumnNode` and chaining stays exact with no recursive generics
  in any type name. It also gives `TextNode` layout, span and paragraph
  properties at once, which single inheritance forbids and which Java needed
  self-typed default methods for. Where two interfaces declare one name — `Size`,
  `Wrap`, `Tag` — `TextNode` declares an instance method, which always beats an
  extension method, and that is also how `Text.size()` ends up being the font
  size rather than the box size.
- **`number | "auto" | "%"`.** A `Dim` struct with implicit conversions from
  `double` and `string`, so `.Width(100)`, `.Width("50%")` and `.Width(Dim.Auto)`
  are one method. Same trick for grid tracks, font weights and page margins.
  Keyword values are structs with named constants *and* an implicit string
  conversion, which is PHP's backed-enums-or-strings without a second overload.
- **Name collisions.** Factories and classes are split the way Python splits them
  — `Column()` returns a `ColumnNode` — because a static method and a type cannot
  share a name and stay invocable. That defuses everything except `Path`, which
  is `System.IO.Path` and arrives with the implicit usings, so it is `SvgPath`.
  `List` and `Span` survive: the BCL types are generic, and arity distinguishes
  them from the zero-arity factories.
- **Named arguments**, which the shorthands lean on: `.Padding(top: 20, left: 4)`
  fills the missing sides the CSS way, and render config is typed rather than a
  bag.

Mechanically: the IR is written straight to UTF-8 with `Utf8JsonWriter` and
passed as a pinned NUL-terminated buffer, so the document never round-trips
through UTF-16; `LibraryImport` and source-generated JSON keep the assembly
trimmable and NativeAOT-clean.

`dotnet test` runs 38 tests, the last of which is the parity gate: a document
built with the fluent API renders byte for byte identical through this binding
and through `sone-cli`. Left to do is packaging — the native library wants
per-RID packages, because one static Skia is ~17 MB and a single package
carrying every platform would be ~70 MB for every consumer.

---

## Node, Bun, Deno — shipped

napi-rs, published as **`@sonejs/sone`** (`sone` on npm is the TypeScript
engine). One package covers all four runtimes: a prebuilt Node-API addon for
Node, Bun and Deno, and `@sonejs/sone-wasm` for browsers, chosen by the
`browser` export condition. Everything above `ts/binding.ts` is shared, so the
API is identical on all of them.

```ts
import { Column, Font, Row, Text, sone } from "@sonejs/sone";

await Font.load("Inter", "fonts/Inter-Regular.ttf");

const root = Column(
  Text("Hello").size(28).weight("bold"),
  Row(
    Column().bg("lightgreen").size(50).rounded(14),
    Column().bg("salmon").height(50).rounded(14).flex(1),
  ).gap(10),
).gap(20).padding(20).size(420, 300).bg("khaki").cornerRadius(28);

await sone(root).save("card.png", { density: 2 });
```

This is the one binding where the "reimplement the builder idiomatically" rule
does not apply, because the host language *is* the reference language.
`ts/core.ts` and `ts/ir.ts` are the TypeScript engine's own files, vendored with
only their import lines changed — `ir.ts` already emitted `sone: 1` documents.
`__test__/ir-parity.test.ts` builds thirteen trees with both copies and asserts
the documents are identical, which turns "did the vendoring drift?" into a
failing test rather than a rendering difference found months later.

Node-specific decisions:

- **Rendering goes through `AsyncTask`,** not `#[napi] async fn`. It is
  CPU-bound, so the libuv threadpool is the right pool, and sone v2's API was
  already promise-returning — the same signatures now also keep the event loop
  free.
- **The error class travels in the message.** Node-API fills a thrown error's
  `code` from the napi status and there is no way to set a custom one, so the
  addon prefixes `sone:ir:` / `sone:asset:` / `sone:render:` and `ts/errors.ts`
  strips it while constructing `IrError` / `AssetError` / `RenderError`.
- **Remote and browser assets are resolved before the engine sees them.**
  `Assets::read` refuses `http(s)` by design, and the WebAssembly build has no
  filesystem at all. `ts/assets.ts` walks the IR once, fetches what the engine
  will not, and rewrites the source to an `asset:` handle — one place, both
  backends.
- **Deno needs `--allow-ffi` and `--node-modules-dir=auto`.** It also does not
  run install scripts, which is fine here: napi-rs resolves the platform
  package through `optionalDependencies`, with no `postinstall`.
- **Separate cargo workspace**, for the reason PyO3 needs one — see
  [porting notes](porting-notes.md).

Not carried over from sone v2: the exports that hand out CanvasKit objects
(`SoneCanvas`, `SoneImage`, `SonePath`, `configureSkia`, `.canvas()`), and the
canvas-returning `render` / `renderPages` / `renderWithMetadata`. The full table
is in `bindings/node/README.md`. `yolo.ts`, `qrcode.ts` and the `shiki` subpath
are pure TypeScript over the same data and can follow.

---

## PHP — built, not yet packaged

FFI over `include/sone.h`, so `composer require` with no build step. Two
practical wrinkles: PHP's FFI parser cannot digest `#include <stdint.h>`, so ship
a preprocessed header with plain typedefs; and FFI is enabled in CLI but gated
behind `ffi.enable=preload` in typical production configs.

```php
use function Sone\{Column, Row, Text, Span};
use Sone\{Font, Sone};

Font::load('Inter', 'fonts/Inter-Regular.ttf');

$root = Column(
    Column()->flex(1)->cornerRadius(20)->cornerSmoothing(0.7)->bg('white'),
    Row(
        Column()->bg('lightgreen')->size(50)->borderRadius(14),
        Column()->bg('salmon')->height(50)->borderRadius(14)->flex(1),
    )->gap(10),
)->gap(20)->padding(20)->size(420, 300)->bg('khaki');

Sone::render($root)->save('card.png', density: 2);
```

Taken: named arguments (`->padding(top: 20, right: 16)`), spread for generated
children (`Table(...array_map(...))`), and backed enums accepted alongside
strings. Every setter returns `static`, so a chain keeps its concrete type.

Two things the implementation settled that the sketch did not:

- **`list` is a reserved word**, so it could not be a class name or a function
  name. The factory is `BulletList()` and the class is `ListNode`.
- **Traits collide on `size`** — `LayoutProps` has the box, `SpanStyleProps` the
  font — and `insteadof` is exactly the tool for it. `Text` resolves it toward
  the font size, matching the TypeScript API, and keeps the box as `boxSize()`.

`php tests/run.php` runs 35 tests including the parity gate, on a plain-PHP
runner so the suite needs no composer install. Left to do is packaging the
native library into the composer package.

---

## Ruby — working, not yet packaged

A block DSL rather than a fluent chain. Ruby is the one language here where the
tree can be written as nested blocks with no punctuation ceremony at all, and
where the structure of the code is the structure of the document:

```ruby
require "sone"

root = Sone.column do
  gap 20
  padding 20
  size 420, 300
  bg "khaki"
  corner_radius 28

  column { flex 1; corner_radius 20; corner_smoothing 0.7; bg "white" }

  row do
    gap 10
    column { bg "lightgreen"; size 50; border_radius 14 }
    column { bg "salmon"; height 50; border_radius 14; flex 1 }
    column { bg "orange"; size 50; border_radius 14 }
  end
end

Sone.render(root).save("card.png", density: 2)
```

Properties and children are the same kind of call — `gap 20` sets a property,
`row do … end` appends a child — so a block reads top to bottom as "what this
box is, then what is in it". `do…end` for multi-line, `{ … }` with semicolons
for a one-liner. This is the shape Rake, RSpec and Sinatra taught every Ruby
reader, and unlike the fluent form it needs no `include` at the top level: the
old design put `Column()` and friends on `Object`, and this puts nothing
anywhere.

Method-for-method parity with `core.ts` survives — every name in a block is a
`core.ts` method in snake_case, with camelCase aliases — it is only the shape
that changes.

### What the block form buys

**Generated children are just Ruby.** This is the real win, and it is where the
fluent form in every other binding needs a splat or a `map`:

```ruby
table do
  spacing 0, 8
  rows.each do |record|
    table_row do
      record.cells.each { |cell| table_cell { text cell } }
    end
  end
  table_row { table_cell { text "No results" } } if rows.empty?
end
```

**Text reads as text.**

```ruby
text do
  font "Inter"
  size 28
  line_height 1.4
  align :justify

  content "Hello "
  span("world") { weight :bold; color "salmon" }
end
```

### The two things to get right

**`instance_eval` moves `self`, so the caller's `@ivars` and helper methods stop
resolving inside a block.** Local variables still close over fine, which is what
catches people out — `column { bg colour }` works and `column { bg @colour }`
silently reads the builder's `@colour`. The standard Ruby answer is to yield the
builder when the block takes an argument, and only `instance_eval` when it does
not:

```ruby
column do |c|          # self is still your object here
  c.bg @colour
  c.size 50
end
```

Both forms must work, decided per block by `block.arity`.

**Locals shadow the DSL.** `size = 50` above a block turns every later `size 50`
into a parse error, because Ruby resolves the bare name to the local. Nothing
can be done about it in the library; it belongs in the docs next to a
recommendation to name locals for what they hold. `self.size 50` is the escape
hatch.

A no-argument call reads rather than writes — `gap` returns the current value,
`gap 20` sets it — so a block can branch on what it has already been given.

### The rest

Symbols for keywords (`:space_between` → `"space-between"`), camelCase aliases
next to every snake_case name, `to_h` exposing the IR, and every property setter
returning `self` so the fluent form still works for one-liners where a block is
more ceremony than the thing it wraps.

Built on the `ffi` gem over `include/sone.h` rather than magnus + rb-sys: the C
ABI is the whole contract, so `gem install sone` needs no Rust toolchain and no
build step — which is the same trade the PHP binding will make. `rake test` runs
42 tests including the parity gate. Left to do is packaging the native library
into the gem; today it is found by walking up to a checkout, or through
`SONE_NATIVE_LIBRARY`.

Two details the implementation settled that the sketch above did not:

- **`end` is a keyword**, so the trailing inset is `inset_end`. Ruby does allow
  a keyword-named method with an explicit receiver, so `self.end 8` also works —
  it is only the bare call that cannot parse.
- **`page_break` is both a property and a factory** in the TypeScript API. As
  bare calls on the same object they collide, so the factory is `page_break!`
  and the property keeps the plain name.

---

## JVM — built, runs on Android

Java and Kotlin share one native layer. Panama (`java.lang.foreign`) with
`jextract` generating bindings from `include/sone.h` needs no glue at all, and
puts the floor at Java 22 — realistically Java 25 LTS.

Java, with constructors (chaining off `new` needs no extra parens):

```java
var root = new Column(
    new Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white"),
    new Row(
        new Column().bg("lightgreen").size(50).borderRadius(14),
        new Column().bg("salmon").height(50).borderRadius(14).flex(1)
    ).gap(10)
).gap(20).padding(20).size(420, 300).bg("khaki");
```

Java forces three decisions: a `Dim` type for `number | "auto" | "%"`, arity
overloads in place of named arguments, and self-typed interfaces with default
methods so `Text` can carry layout, span and paragraph properties under single
inheritance. `List` and `Path` collide with the JDK — rename those two to
`Bullets` and `SvgPath`.

Kotlin gets a DSL receiver, which also sidesteps the Compose name clash by
keeping `Column`/`Row`/`Text` inside the block:

```kotlin
val png = sone {
    Column {
        gap = 20
        padding(20)
        Row { gap = 10; Column { bg("salmon"); size(50) } }
    }
}.png(density = 2)
```

The core ended up in **Java**, with the Kotlin DSL as a layer on top — the
reverse of what this section first said, for the reason it gave. DSL receivers
do not translate to Java, so keeping the fluent surface in Java is what gets
both languages a first-class API; Kotlin then gets `dev.sone.dsl` for free.

The cost is that `@DslMarker` cannot be applied: it has to annotate the receiver
type, and these receivers are Java classes. A nested block can still reach the
enclosing builder by accident, and Kotlin's labelled `this` — `this@Column` — is
how you reach it on purpose.

Panama is bound by hand rather than through `jextract`: sixteen functions and
three structs is less code than the generator's plumbing, and it keeps the build
to a plain `mvn package`. Java 22 is the floor, because that is where Panama is
final. `layoutJson()` returns a `String` rather than a parsed tree, so the
artifact does not hand every consumer a Jackson version to reconcile.

### Android

Android has no `java.lang.foreign` at any version — ART does not implement
Panama, and it cannot be desugared because it needs VM support. So the binding
splits into `sone-core` (the builder, no native code, Java 17), `sone-panama`
(desktop) and `sone-jna` (Android), with `Engine` choosing a backend at runtime
behind `dev.sone.Backend`.

JNA rather than a JNI shim, because JNA reuses `include/sone.h` unchanged and a
JNI layer would have been a second ABI to keep in step. The per-call overhead is
irrelevant at this granularity: one render is one call carrying a whole
document.

The JNA backend also runs on a desktop JVM, which is how the Android path is
tested without a device — `BackendTest` asserts the two backends produce
byte-identical PNGs and identical layout JSON. Both Android artifacts were
checked through `d8 --min-api 21`.

`mvn test` runs 50 tests: 31 Java, 6 Kotlin, and 13 across both backends,
including the parity gate.

---

## Dart — built, runs on Flutter and Android

`dart:ffi` over `include/sone.h`, and eventually native assets
(`hook/build.dart`) to bundle the library so consumers need no toolchain.

Flutter-style named arguments, which is what a Dart reader expects:

```dart
import 'package:sone/sone.dart' as s;

final root = s.Column(
  gap: 20, padding: 20, width: 420, height: 300, bg: 'khaki', cornerRadius: 28,
  children: [
    s.Column(flex: 1, cornerRadius: 20, cornerSmoothing: 0.7, bg: 'white'),
    s.Row(gap: 10, children: [
      s.Column(bg: 'lightgreen', size: 50, cornerRadius: 14),
      s.Column(bg: 'salmon', height: 50, cornerRadius: 14, flex: 1),
    ]),
  ],
);
```

Every property is a parameter, which means ~75 of them on every container
constructor. They are generated from the same table the setters come from
(`tool/generate_nodes.py`), so the two forms cannot drift.

Cascades are the second shape, and they are not a fallback — `..` evaluates to
the receiver, so the setters return `void` and none of the self-type machinery
Java and Kotlin needed exists here. Named arguments call straight into those
setters. Two things only cascades can say:

- **The order filters apply in.** Named arguments apply them in declaration
  order; `..grayscale(0.5)..blur(4)` is how you choose.
- **An explicit null.** A null named argument means "unset"; the explicit null
  that clears a decoration colour to the text colour is `..underlineColor()`.

Collection-`for` and collection-`if` inside `children:` are the best answer to
generated content across every language here, and the import prefix handles the
Flutter widget name collision in one line. FFI calls block the isolate, so a
Flutter app should render inside `Isolate.run` — with its own `Engine`.

Three names moved: `List` → `Bullets` and `Path` → `SvgPath` (`dart:core` and
`dart:ui` own the originals), and the font size is `fontSize` on the mixin,
because Dart mixins linearize and two of them cannot declare `size` with
different signatures. `Text` overrides `size` to mean the font size anyway, so
`s.Text('Hi', size: 28)` is the font size and `width`/`height` are the box —
that override is load-bearing, not decoration.

### Flutter

`bindings/flutter` is a separate FFI plugin, `sone_flutter`, rather than a
`flutter:` section on the pure-Dart package — a plugin cannot be used from
`dart run` or `dart test`, and the engine has to stay usable from a command
line. The plugin depends on the Dart package, carries the Android `.so` files in
`jniLibs`, and adds the two things a Flutter app needs:

- **Fonts from the asset bundle**, because Skia carries no system fonts on
  Android either.
- **`pngAsync` / `pdfAsync` / `pagesAsync`**, which render on a background
  isolate with an engine of their own. FFI blocks whichever isolate calls it, so
  the synchronous API drops frames on the UI one. Only sendable values cross:
  the document as JSON, and fonts as bytes — an `Engine` owns a native pointer
  and cannot be sent.

Verified end to end on an Android arm64 emulator: font out of the asset bundle,
layout and raster through the native engine, a PNG and a real PDF back.

There is no CMake build in the plugin. Skia takes an hour to compile from
source, so `tools/build-android.sh` cross-compiles ahead of time and Gradle only
packages the result — about a minute, because the feature set is chosen to match
a published prebuilt.

`dart test` runs 33 tests including the parity gate. Left to do is native
assets, so consumers need no toolchain, and pointing the plugin at the Swift
XCFramework for iOS.

---

## Swift — built, runs on macOS, iPhone and iPad

A result builder for children and modifiers for properties — the shape every
Swift reader already knows from SwiftUI.

```swift
import Sone

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

Swift answers three of the four decisions the JVM struggled with, for free:

- **`Self`** comes from class-constrained protocol extensions, so chaining keeps
  the concrete type with no recursive generics anywhere.
- **The length union** collapses into literal conformances —
  `ExpressibleByIntegerLiteral`, `ByFloatLiteral`, `ByStringLiteral` on `Dim` —
  so `.width(100)`, `.width("50%")` and `.width(.auto)` are one method. Same for
  grid tracks, font weights and page margins.
- **Protocol composition** gives `Text` all three property sets with no
  single-inheritance fight and no name collision to resolve.

`@resultBuilder` supplies the fourth thing every binding needs: `if` and `for`
inside the children block, plus a second builder for paragraph content, so a
`Text { }` block takes bare strings and `Span`s as statements.

`List` and `Path` are SwiftUI's and Foundation's, so they are `Bullets` and
`SvgPath`, consistent with the JVM and .NET bindings.

Worth noting how it links: the `CSone` system-library target supplies only the
struct layouts, and the functions come from `dlsym` at runtime. That leaves no
link-time dependency, so `swift build` needs no `-L` and no unsafe flags — which
is what keeps the package consumable as a dependency later.
`Sources/CSone/include/sone.h` is a symlink to the canonical header, so the
declarations cannot drift.

### Apple platforms

The library ships as `Sone.xcframework` with three slices — macOS arm64, iOS
arm64, and the arm64 iOS simulator — built by `tools/build-apple.sh` in about a
minute from published prebuilts. iPadOS is iOS, so an iPad app needs nothing
extra.

Static, not dynamic: iOS will not load an arbitrary dylib from a package, and
one linking model for every platform beats two. That is also why the Swift layer
calls the C functions directly rather than resolving them with `dlsym` — a
static archive only contributes the object files something references, so late
binding would let the linker drop every symbol first.

The Apple slices are built without `embed-freetype`, because that is what the
published prebuilt has; Skia then rasterizes glyphs through the platform font
engine. On macOS this makes no difference the parity gate can see — the same
document still comes out byte for byte identical to `sone-cli`.

Verified by running the engine inside an iPhone 16 and an iPad Pro simulator:
font registered, PNG rendered, same byte count on both.

`swift test` runs 30 tests on macOS including the parity gate, which skips
itself on iOS because `Process` does not exist there. tvOS, watchOS and visionOS
are not built — rust-skia publishes no prebuilt for them.

---

## Browser — shipped

`wasm32-unknown-emscripten`, published as **`@sonejs/sone-wasm`**: engine only,
no builder, consumed by `@sonejs/sone` through its `browser` export condition.
Roughly 6 MB of `.wasm`, about 2.5 MB over the wire, dominated by the ICU table
`textlayout` links unconditionally. Single-threaded, so no `SharedArrayBuffer`
and no cross-origin isolation to arrange.

**napi-rs's own WebAssembly target cannot be used for this.** napi-rs v3 targets
`wasm32-wasip1-threads`; `skia-safe` only builds for
`wasm32-unknown-emscripten`. They are mutually exclusive, so the browser build
is a second artifact rather than a second napi-rs target.

Three things about the build are worth knowing before touching it; all three
cost an afternoon to discover and are recorded in
[porting notes](porting-notes.md):

- Skia is compiled **from source**, and there is no way around it. The published
  emscripten prebuilt uses emscripten's legacy JavaScript exception handling,
  current Rust emits `-fwasm-exceptions` for this target, and the escape hatch
  (`-Zemscripten-wasm-eh=false`) has been removed. Independently,
  `skia-bindings` 0.99 cannot consume that prebuilt anyway. It is not slow —
  about two minutes, because the wasm Skia build is far lighter than a desktop
  one.
- The crate **must not depend on `sone-ffi`**, even though its C ABI is the
  right shape. `sone-ffi` declares `crate-type = ["cdylib", …]` and cargo builds
  every declared type, so depending on it makes rustc link a cdylib — which on
  emscripten means `-sSIDE_MODULE=2`, a PIC link, and non-PIC Skia. The wasm
  crate carries its own thin layer instead.
- The ABI is **pointer + length, no structs, no out-parameters**, unlike
  `include/sone.h`. Buffers are opaque handles read through accessor calls, so
  JavaScript never has to know a Rust struct's layout, and two exported
  allocator functions mean the glue never needs emscripten's `malloc`.

Both engines are held to each other by `bindings/node/__test__/wasm.test.ts`:
layout trees identical field for field, every non-text primitive pixel-identical
at density 2, and text bounded rather than exact — glyphs go through CoreText on
macOS and Skia's bundled FreeType here.
