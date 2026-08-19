# sone

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

This is the monorepo for the native engine and every language binding. The
TypeScript engine that started it lives at
[seanghay/sone](https://github.com/seanghay/sone) and remains the behavioural
reference; it will move into `packages/` in time.

```
crates/          the Rust engine
  sone           the Rust-facing fluent builder — no FFI, no IR string
  sone-core      no Skia dependency — IR, CSS parsers, compile, taffy layout,
                 text engine, grid/table/list, pagination, draw traits
  sone-skia      Skia backend — painter, fonts, shaping, images, exports
  sone-ffi       C ABI (cdylib + staticlib)  →  include/sone.h
  sone-cli       `sone render | dump-layout | dump-metadata`
  sone-goldens   parity harnesses: raster dssim diff, numeric layout diff
bindings/        one directory per language
  python         PyO3 + maturin, abi3 wheel                        ready
  csharp         P/Invoke over include/sone.h                      built
  ruby           ffi gem over include/sone.h, block DSL            built
  node           napi-rs — Node, Bun, Deno   @sonejs/sone          ready
  php            FFI over include/sone.h                           built
  jvm            Java over Panama · Kotlin DSL · Android via JNA    built
  dart           dart:ffi, named arguments                         built
  flutter        sone_flutter plugin — Android, off-isolate        built
  swift          XCFramework — macOS, iPhone, iPad                 built
  wasm           emscripten — browsers       @sonejs/sone-wasm     ready
packages/        npm workspace — the TypeScript engine lands here
fixtures/        the parity corpus, generated from the TypeScript engine
tools/           sync-fixtures.sh · build-android.sh · build-apple.sh
docs/            architecture · porting-notes · parity · bindings · roadmap · status
```

## Quick start

```bash
cargo test --workspace
cargo run -p sone-cli -- render fixtures/visual/ir/a4-report.json -o /tmp/a4.pdf

# parity against the TypeScript goldens
cargo run --release -p sone-goldens --bin sone-goldens   # raster dssim + HTML report
cargo run --release -p sone-goldens --bin layout-diff    # numeric layout diff
cargo run --release -p sone-goldens --bin render-all     # render every fixture
```

Node, Bun, Deno and the browser:

```bash
npm install
npm run build --workspace @sonejs/sone-wasm   # needs emscripten; browsers only
npm run build --workspace @sonejs/sone
npm test  --workspace @sonejs/sone
```

```ts
import { Column, Font, Row, Text, sone } from "@sonejs/sone";

await Font.load("Inter", "fonts/Inter-Regular.ttf");

const root = Column(
  Text("Hello").size(28).weight("bold"),
  Row(
    Column().bg("salmon").size(50).rounded(14),
    Column().bg("orange").size(50).rounded(14),
  ).gap(10),
).gap(20).padding(20).bg("khaki").cornerRadius(28);

await sone(root).save("card.png", { density: 2 });
```

Python:

```bash
cd bindings/python && maturin develop --release && pytest tests -q
```

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

## Design

Every binding is thin. The fluent builder API is reimplemented per language and
produces the same JSON **IR document**; the native layer is document-in,
bytes-out. That keeps each surface idiomatic — Python gets snake_case aliases,
Kotlin gets a DSL receiver, Dart gets named arguments — while the layout, text
and drawing logic exists exactly once, in Rust.

The Node binding is the exception that proves the rule: there, the host language
*is* the reference language, so `bindings/node/ts/core.ts` is the TypeScript
engine's own builder — vendored rather than rewritten, with a test asserting
both copies still serialize to identical documents.

`sone-core` carries no Skia dependency at all: it talks to `Painter`,
`TextEngine` and `Backend` traits over plain value types, so the engine is
testable without a rasterizer and a second backend stays possible.

## Documentation

| | |
|---|---|
| [architecture.md](docs/architecture.md) | how a document becomes pixels, and why the crates split where they do |
| [porting-notes.md](docs/porting-notes.md) | every non-obvious behaviour, with the symptom that found it — **read before changing layout or text** |
| [parity.md](docs/parity.md) | the four harnesses, how to diagnose a moved golden, how waivers work |
| [bindings.md](docs/bindings.md) | how a binding is built, plus the surface design for each language |
| [roadmap.md](docs/roadmap.md) | what is left, ordered, with entry points |
| [status.md](docs/status.md) | milestone state and where the binary weight is |

## Parity

The Rust engine is held to the TypeScript one by four gates, all runnable
locally and in CI. Current results; every known divergence is explained in
[parity.md](docs/parity.md):

| Gate | Result |
|---|---|
| `cargo test --workspace` | 148 tests |
| raster goldens, dssim ≤ 0.02 | 43 of 53 clean, 10 waived with written reasons |
| computed layout trees, 0.5px | 40 of 56 exact |
| line breaks vs `Intl.Segmenter` | 1937/1937 non-dictionary scripts exact |
| PDF page counts | 3 / 15 / 5, matching, text selectable |

Waivers live in `goldens-waivers.toml` and each one carries a reason.

## Licence

Apache-2.0.
