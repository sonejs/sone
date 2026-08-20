# Status

Where the port stands, and what the binary is made of. Architecture is in
[architecture.md](architecture.md), the non-obvious behaviours in
[porting-notes.md](porting-notes.md), the harnesses in [parity.md](parity.md).

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
| M8 language bindings | Python (PyO3) shipped. C#, Ruby, PHP, Dart, JVM and Swift all build and pass their parity gates. Mobile: Dart runs on Android through the `sone_flutter` plugin, the JVM runs on Android through `sone-jna`, and Swift runs on iPhone and iPad through an XCFramework. Packaging (NuGet, gem, pub, Maven Central, SPM release) is the remaining work. Known gap: the JVM binding crashes on Windows when laying out text — see bindings.md. |
| GPU backend | not started — the cargo features exist, the surface path does not; `--backend gpu` warns and falls back to CPU |

### Verification gates

Detail and diagnosis in [parity.md](parity.md).

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
