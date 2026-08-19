# Roadmap

What is left, roughly in the order I would do it, with an entry point for each.

---

## Close the last parity gaps

Two of the four waivers are worth another attempt; the other two probably are
not.

### Table cell cross-sizing — worth fixing

The highest-value one, because it also corrupts `doc.pdf`'s extracted text
(words split mid-token, `Activ e`). taffy derives a flex item's hypothetical
cross size from its natural content width, so a cell whose text is wider than
its column wraps before the column minimum widens it.

Already tried and rejected: setting `size.width` on the cell, and setting
`flex_basis` — both made every table dramatically worse (heights 10× too large),
which suggests some `span_w` values are zero on that path. Worth instrumenting
`apply_table_layout` in `layout/table.rs` before trying a third variation.

A more promising angle: lay each cell out once at its final column width in a
detached tree, cache the resulting height, and set `min_size.height` from that —
i.e. take over the row-height decision entirely rather than nudging taffy's.

Entry: `crates/sone-core/src/layout/table.rs`, gate with
`cargo run -p sone-goldens --bin layout-diff -- table`.

### Flex free-space distribution with `textWrap: "balance"` — worth understanding

Yoga splits a row's free space 526/365 between two `flex(1)` columns; taffy
splits it evenly. The balanced paragraph is identical, so this is purely how
Yoga resolves flex lines when items have measure functions. Understanding it may
fix a class of layouts rather than one fixture.

Entry: `cargo run -p sone-goldens --bin layout-diff -- text-wrap-balance-1`,
then read Yoga's `YGDistributeFreeSpaceSecondPass`.

### Khmer line breaking — probably not fixable

Skia's ICU dictionary differs from V8's; 36/108 agree. Switching to ICU4X gets
42/108 for 2.3 MB. Neither reaches parity. Realistic options: accept it, or
extract V8's dictionary and ship it — a big lift for a small gain.

### Remote assets — working as designed

`text-2` stays skipped.

---

## M5: metadata and datasets

`dump-metadata` emits the node tree and text boxes at node/line/word
granularity, which is enough for inspection but is **not** the TypeScript
`metadata.ts` shape, and `yolo.ts` is not ported at all.

To finish: port `yolo.ts` (561 lines — YOLO and COCO export), and make
`metadata.rs` match `SoneMetadata` field for field so the M5 gate (JSON
deep-equality against the TypeScript output, coordinates within 0.5px) can
actually run.

One known TypeScript quirk to decide about: `createTextRuns` applies the
paragraph-width expansion a second time when it runs after `drawOnCanvas`, so
metadata coordinates differ from draw coordinates when a text node has
horizontal padding. The Rust code computes it cleanly. Reproducing the bug for
byte-equality would be a poor trade.

Entry: `crates/sone-core/src/metadata.rs`.

---

## M7: browser — done

`wasm32-unknown-emscripten`, shipping as `@sonejs/sone-wasm` and consumed by
`@sonejs/sone` through its `browser` export condition.

Gate met: `bindings/node/examples/browser/index.html` renders a Khmer card in
Chrome — engine ready in 42 ms, 14.6 KB PNG. Binary size **6.1 MB** of `.wasm`,
about 2.5 MB over the wire, still dominated by the ICU table.

Skia is built from source for this target; `bindings/wasm/build.sh` and the
emscripten section of [porting-notes.md](porting-notes.md) explain the four
reasons the published prebuilt cannot be used. It takes about two minutes.

Left over: `render_pages` on a worker pool, and a `tiny-skia` backend if the
6 MB ever needs to be 600 KB.

---

## M8: the remaining bindings

Design and syntax for each is in [bindings.md](bindings.md). Node is done —
`@sonejs/sone` covers Node, Bun, Deno and the browser from one package — so
order the rest by demand.

Every one of them should ship with the CLI-parity test the Python binding has.

---

## GPU backend

Cargo features exist (`gpu-metal`, `gpu-gl`, `gpu-vulkan`, `gpu-d3d`); the
surface-creation path does not. `sone-cli --backend gpu` warns and falls back.

The `Backend::render` seam already hides surface creation from core, so this is
contained to `sone-skia`. Determinism policy is already decided: goldens stay
CPU-only, GPU gets per-platform smoke tests asserting dssim ≤ 0.03 against the
CPU render of the same document.

---

## Smaller things

- **`packages/`** — the TypeScript engine moves in. The npm workspace now
  exists at the repository root and already covers `bindings/node` and
  `bindings/wasm`; `packages/sone` joins it. When it does, it can import the
  builder from `bindings/node/ts` instead of the two copies living side by side
  — `__test__/ir-parity.test.ts` is what makes that swap safe to attempt.
- **Font licensing** — `fixtures/font/GoogleSans-VariableFont` is not
  redistributable. Swap it for an open face before this repo goes public, and
  regenerate the affected goldens.
- **Parallel pages** — `render_pages` is sequential. The engine is `Send + Sync`
  and rayon is already a dependency of the goldens harness; multi-page PDF at
  N× cores was one of the stated reasons for the port.
- **Shiki equivalent** — `syntect`, optional and last. `shiki.ts` is 128 lines
  of pure span construction.
- **`--strict` coverage** — implemented for IR parsing; the "invalid input is an
  error in strict, a warned default otherwise" rule is not applied consistently
  in the CSS parsers, which still fall back to black.
- **Inset and spread box shadows**, and **corner radius on non-uniform borders**
  — on the original improvements list, not yet done.
