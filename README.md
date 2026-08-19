# sone

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

This is the monorepo for the native engine and every language binding. The
TypeScript engine that started it lives at
[seanghay/sone](https://github.com/seanghay/sone) and remains the behavioural
reference; it will move into `packages/` in time.

```
crates/          the Rust engine
  sone-core      no Skia dependency — IR, CSS parsers, compile, taffy layout,
                 text engine, grid/table/list, pagination, draw traits
  sone-skia      Skia backend — painter, fonts, shaping, images, exports
  sone-ffi       C ABI (cdylib + staticlib)  →  include/sone.h
  sone-cli       `sone render | dump-layout | dump-metadata`
  sone-goldens   parity harnesses: raster dssim diff, numeric layout diff
bindings/        one directory per language
  python         PyO3 + maturin, abi3 wheel                        ready
  node           napi-rs                                           planned
  php            FFI over include/sone.h                           planned
  ruby           magnus + rb-sys                                   planned
  jvm            Java and Kotlin over Panama                       planned
  dart           dart:ffi + ffigen                                 planned
  wasm           emscripten                                        planned
packages/        npm workspace — the TypeScript engine lands here
fixtures/        the parity corpus, generated from the TypeScript engine
tools/           sync-fixtures.sh
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
