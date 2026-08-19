# The Rust engine

A native port of the sone layout engine: `taffy` for flexbox, `skia-safe` for
rasterization, PDF and SVG. The TypeScript package under `../packages/sone/src` is unchanged
and remains the behavioral reference; the two engines exchange documents through
a JSON **IR** and are compared by two harnesses (numeric layout diff, perceptual
raster diff) against the same committed goldens.

```
crates/
  sone-core     no skia dependency — IR, CSS parsers, compile, taffy adapter,
                text engine, grid/table/list layout, pagination, draw traits
  sone-skia     skia-safe backend — painter, fonts, shaping, images, exports
  sone-ffi      C ABI (cdylib + staticlib), header at ../include/sone.h
  sone-cli      `sone render` / `dump-layout` / `dump-metadata`
  sone-goldens  dev harness: raster dssim diff + numeric layout diff
bindings/
  python        Python extension (PyO3) + the fluent builder API in Python
```

## Quick start

```bash
cargo test --workspace

# Regenerate the IR + layout corpus from the TypeScript engine (only needed
# after a fixture changes).
tools/sync-fixtures.sh ../sone

cargo run -p sone-goldens --bin sone-goldens      # raster diff + HTML report
cargo run -p sone-goldens --bin layout-diff       # numeric layout-tree diff

cargo run -p sone-cli -- render fixtures/visual/ir/a4-report.json -o /tmp/a4.pdf

# Render every fixture with the Rust engine into target/renders/ (JPEG at
# density 2, PDF for the paginated ones) plus a side-by-side index.html.
cargo run --release -p sone-goldens --bin render-all
```

`target/renders/` is gitignored: it is generated output, and `fixtures/visual/*.jpg`
stays the TypeScript golden corpus the harnesses diff against.

`cargo run -p sone-cli -- --help` lists the commands and options.

## Python binding

`bindings/python` carries the same fluent API as the TypeScript package, so a
tree transliterates one for one:

```python
from sone import Column, Row, Text, Font, sone

Font.load("Inter", "fonts/Inter-Regular.ttf")

root = (
    Column(
        Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white"),
        Row(
            Column().bg("lightgreen").size(50).borderRadius(14).borderColor("teal").borderWidth(2),
            Column().bg("salmon").height(50).borderRadius(14).flex(1),
            Column().bg("orange").size(50).borderRadius(14),
        ).gap(10),
    )
    .gap(20).padding(20).size(420, 300).bg("khaki")
    .cornerRadius(28).borderColor("chocolate").borderWidth(4).rotate(20)
)

sone(root).save("card.png", density=2)
```

The builders live in ``bindings/python/python/sone/_nodes.py`` and only produce IR; the compiled
module is document-in, bytes-out. Method names match TypeScript exactly, with a
snake_case alias generated for each.

```bash
cd bindings/python
maturin develop --release   # or: maturin build --release
pytest tests -q             # 40 tests
```

It is a workspace of its own, because a Python extension needs unwinding panics
for PyO3 to raise exceptions while the main workspace aborts — the right setting
for its C ABI. `abi3-py39` means one wheel covers CPython 3.9 and up.

## Current state

| Milestone | Status |
|---|---|
| M0 scaffolding + golden harness | done — skia-safe 0.99 builds from prebuilts on macOS; IR dumper, raster harness with waivers, numeric layout diff |
| M1 IR, compile, box layout, basic draw | done |
| M2 text engine | done — greedy / Knuth-Plass / balanced breakers, justify, maxLines + ellipsis, tab stops and leaders, indents, decorations, per-span gradients, stroke, drop shadows, bidi subset, orientation, autofit, clip-image |
| M3 full draw ops + hand-rolled layouts | done — photos, paths, clip groups, box shadows, CSS filters, transforms, group opacity, grid, table spans, list markers |
| M4 exports + pagination | done — PNG/JPEG/WebP/raw/PDF/SVG, page breaks, header/footer bands, `{pageNumber}`/`{totalPages}` tokens, multi-page PDF |
| M5 metadata | partial — `dump-metadata` emits the node tree and text boxes at node/line/word granularity; the YOLO/COCO exporters from `yolo.ts` are not ported |
| M6 FFI + CLI | done — opaque `SoneEngine*`, committed `include/sone.h`, C smoke test asserting byte-identical output to the CLI |
| M7 browser (emscripten) | not started |
| M8 language bindings | Python done (`bindings/python`, PyO3 + maturin, abi3 wheel); Node, C# and PHP not started |
| GPU backend | not started — the cargo features exist, the surface path does not; `--backend gpu` warns and falls back to CPU |

### Verification gates

- `cargo test --workspace` — 145 tests: CSS parsers, squircle, IR round-trip,
  taffy prop mapping, layout, breakers on a mock text engine, pagination,
  draw-op ordering on a recording painter, skia capability probes, C ABI.
- `cargo run -p sone-goldens --bin sone-goldens` — 53 fixtures rendered at
  density 2 and compared with dssim against the TypeScript JPEGs. 43 pass under
  the 0.02 default; 10 carry written waivers in `goldens-waivers.toml`.
- `cargo run -p sone-goldens --bin layout-diff` — computed layout trees compared
  numerically at 0.5px. 40 of 56 fixtures match exactly; the 16 that differ are the divergences listed below.
- `cargo test -p sone-skia --test break_parity` — line breaks against a corpus
  of every string in the fixtures. Non-dictionary scripts match `Intl.Segmenter`
  exactly (1937/1937); Khmer is ratcheted, see below.
- `cargo test -p sone-skia --test measure_parity` — glyph advance widths and
  font metrics against values dumped from the TypeScript engine.
- PDF: `a4-report`, `doc` and `feature-showcase` produce the same page counts as
  the TypeScript engine (3 / 15 / 5) with selectable text; `a4-report`'s
  extracted text is identical after whitespace normalization.


## Binary size

The release CLI is **17.6 MB**, down from 24.5 MB. What it is made of:

| Component | Size | Notes |
|---|---|---|
| Skia's ICU data table | 9.98 MB | unavoidable, see below |
| Skia C++ core | 3.6 MB | rasterizer, paths, filters |
| C runtime + codecs | 1.3 MB | |
| sone crates | 0.73 MB | |
| HarfBuzz | 0.60 MB | complex-script shaping |
| Skia ICU code | 0.31 MB | |
| libwebp / libpng / libjpeg | 0.44 MB | image decode and encode |
| Skia SVG + PDF modules | 0.29 MB | `.svg()` and `.pdf()` output |

What was removed:

- `strip = "symbols"`, `lto = "fat"`, `panic = "abort"` on the release profile —
  **2.4 MB**. Aborting on panic is also the right FFI behavior: unwinding across
  the C boundary is undefined.
- **ICU4X (2.3 MB).** Word segmentation now runs on Skia's bundled ICU, which
  the binary pays for anyway. UAX#29 scripts use `unicode-segmentation` (already
  a dependency for grapheme clusters) and still match `Intl.Segmenter` exactly;
  Khmer agreement moved from 42/108 to 36/108, which is the recorded waiver.
- **`regex` (0.46 MB).** The two patterns it served — URL and phone-separator
  protection in `linebreak.rs` — are hand-rolled scanners with their own tests
  asserting the same matches.
- **taffy's `grid`, `block_layout`, `calc`, `content_size` and
  `detailed_layout_info` features.** sone hand-rolls its own grid, so only
  flexbox is used.
- **skia-safe's default features** are pinned to an explicit list, so a new
  default cannot silently grow the binary.

Debug builds went from 41 MB to **25.4 MB** via `debug = 1` plus optimized
dependencies; they link the same static Skia, so that is close to the floor.

### Why the 10 MB ICU table stays

`skia_use_icu` is unconditional once `textlayout` is on, and skia-safe's
`embed-icudtl` feature is an empty no-op in 0.99 — toggling it changes nothing.
Dropping it would mean building Skia from source with `skia_use_libgrapheme`,
losing the prebuilt binaries, the dictionary segmentation Khmer needs, and the
bidi implementation. It is the price of correct international text.

## Known divergences

Each is reproducible through the harnesses above and recorded, with a reason, in
`goldens-waivers.toml`.

1. **Khmer line breaking.** Dictionary scripts are segmented by Skia's bundled
   ICU, whose Khmer dictionary differs from the one V8 ships behind
   `Intl.Segmenter`: 36 of 108 Khmer strings in the corpus break identically.
   Every other script matches exactly. ICU4X agreed on 42/108 but cost 2.3 MB of
   duplicate data, so Skia's ICU — already linked — is used instead.
2. **Table cell cross-sizing.** taffy derives a flex item's hypothetical cross
   size from its natural content width, so a cell whose text is wider than its
   column wraps before the column minimum widens it. Yoga re-measures the cell
   at its final width. Affects `table-2`, `table-span-row`, `text-1`.
3. **Flex free-space distribution with `textWrap: "balance"`.** Yoga and taffy
   split a row's free space differently between two `flex(1)` columns. The
   balanced paragraph itself is identical; only its host column's width differs.
4. **Remote assets.** The engine never fetches over the network during a render.
   Callers download and register bytes as `asset:<name>`; `text-2` is skipped.

## Deliberate improvements over the TypeScript engine

Each is additive and covered by tests.

- **Radial gradients** are implemented. The TypeScript engine parses
  `radial-gradient` / `repeating-radial-gradient` and then skips them.
- **Odd-length dash arrays** are duplicated the way Canvas2D does, so
  `strokeDashArray(14)` dashes instead of drawing a solid line.
- **Typed props.** CSS colors, gradients, shadows and filters are parsed once at
  the IR boundary into typed values; nothing is re-parsed during layout or draw.
- **`Send + Sync` engine.** Fonts and images are shared immutably behind
  `Arc`/`Mutex`, so pages can be rendered in parallel.
- **Errors instead of silent fallbacks.** A single `SoneError` carries a JSON
  pointer for IR failures and maps onto CLI exit codes (2 IR / 3 asset / 4
  render) and FFI status codes.

## Notes on the port

- **Yoga defaults are shimmed** at the taffy boundary: column direction,
  `flex-shrink: 0`, `align-content: flex-start`, and Yoga's `flex` shorthand
  (grow / shrink / basis) — see `layout/style.rs`.
- **Pixel rounding is Yoga's, not taffy's.** taffy's own rounding is disabled;
  `layout/engine.rs` reimplements `roundValueToPixelGrid`, including the
  round-outward rule for nodes with a measure function. This alone removed most
  of the 1px differences in the layout diff.
- **Variable fonts pin every axis.** CanvasKit defaults unspecified variation
  axes; bare Skia clamps them to the axis minimum. Leaving `opsz` unset made
  Google Sans measure 2% wide. See `sone-skia/src/text.rs`.
- **taffy's measure contract** hands the callback an *inner* available space but
  an *outer* `known_dimensions`. Wrapping reads the available space.

## Where the fixtures come from

The corpus in `fixtures/` is generated by the TypeScript engine at
[seanghay/sone](https://github.com/seanghay/sone), which added one file to its
public surface — `src/ir.ts`, a pure serializer producing the IR document — plus
dumpers under `test/visual/` (`dump-ir.ts`, `dump-layout.ts`, `dump-breaks.ts`).
Nothing in its rendering path changed.

`tools/sync-fixtures.sh <path-to-sone-checkout>` re-runs those dumpers and copies
the results here, goldens and fonts included.
