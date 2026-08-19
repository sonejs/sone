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

- **Fluent API matching `core.ts` method for method.** Names may be adapted to
  the host convention, but with an alias back to the TypeScript spelling so
  examples transfer.
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

## Node — planned

napi-rs, published as **`@sonejs/sone`** (`sone` on npm is the TypeScript
engine). This is also the future native fast path for that package.

The builder can be lifted almost verbatim from `core.ts`, which is the point:
the TypeScript API is the reference, so the Node binding should be
indistinguishable from it.

---

## PHP — planned

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

Worth taking: named arguments (`->padding(top: 20, right: 16)`), spread for
generated children (`Table(...array_map(...))`), backed enums accepted alongside
strings, and closures for per-item markers.

---

## Ruby — planned

The best syntactic fit of the lot: uppercase method names are legal and
idiomatic — `Integer()`, `Array()` and `String()` work exactly this way — so the
TypeScript names survive verbatim with no collisions and no import ceremony.
Ruby also keeps trailing commas.

```ruby
require "sone"
include Sone::DSL

root = Column(
  Column().flex(1).corner_radius(20).corner_smoothing(0.7).bg("white"),
  Row(
    Column().bg("lightgreen").size(50).border_radius(14),
    Column().bg("salmon").height(50).border_radius(14).flex(1),
  ).gap(10),
)
  .gap(20)
  .padding(20)
  .size(420, 300)
  .bg("khaki")

Sone.render(root).save("card.png", density: 2)
```

snake_case primary with camelCase aliases. Symbols for keywords
(`:space_between` → `"space-between"`), splat and `<<` for generated children,
`to_h` exposing the IR. magnus + rb-sys with precompiled gems via
`rake-compiler-dock`, or the `ffi` gem for a pure-Ruby route.

`include Sone::DSL` at top level defines those methods on `Object`; for
libraries, include into an explicit class or ship it as a refinement.

---

## JVM — planned

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

Use `@DslMarker` so a nested builder cannot silently configure its parent. Write
the core in Kotlin with a Java-facing fluent facade, not the reverse — DSL
receivers do not translate to Java at all.

---

## Dart — planned

`dart:ffi` with `package:ffigen` from `include/sone.h`, and native assets
(`hook/build.dart`) to bundle the library so consumers need no toolchain.

Dart is unusual in having two idiomatic shapes. Flutter-style named arguments:

```dart
import 'package:sone/sone.dart' as s;

final root = s.Column(
  gap: 20, padding: 20, width: 420, height: 300, bg: 'khaki',
  children: [
    s.Column(flex: 1, cornerRadius: 20, bg: 'white'),
    s.Row(gap: 10, children: [
      s.Column(bg: 'salmon', height: 50, cornerRadius: 14, flex: 1),
    ]),
  ],
);
```

Or cascades, which are the interesting option: `..` evaluates to the receiver,
so setters can return `void` and none of the self-type machinery Java and Kotlin
needed is required at all.

Collection-`for` and collection-`if` inside `children:` are the best answer to
generated content across every language here. The import prefix handles the
Flutter widget name collision in one line. FFI calls block the isolate, so a
Flutter app should render inside `Isolate.run` — with its own `Engine`.

---

## Browser — planned

`wasm32-unknown-emscripten` build of skia-safe, following rust-skia's
`wasm-example`. Needs the EMSDK toolchain and `ERROR_ON_UNDEFINED_SYMBOLS=0`.
Same IR-in, bytes-out contract. Binary size will be the interesting number —
see the binary-size section of [status.md](status.md) for where the weight is.

If the target proves too fragile, the TypeScript package remains the browser
story and `sone-core` stays backend-agnostic for a future `tiny-skia` backend.
