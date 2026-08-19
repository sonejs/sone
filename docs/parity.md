# Parity

How the Rust engine is held to the TypeScript one, and how to diagnose a
failure. Four harnesses, ordered by how useful they are when something breaks.

---

## The harnesses

### 1. Layout diff — start here

```bash
cargo run --release -p sone-goldens --bin layout-diff [name-filter]
```

Compares the computed layout tree against `fixtures/visual/layout/*.json`
(dumped from Yoga) at 0.5px, node by node, and prints the first six differing
nodes with a path:

```
DIFF text-wrap-balance-1  (18 nodes differ)
       root/1/0 (column) width: ts=526 rust=446
       root/1/0/0 (text) width: ts=486 rust=406
```

This is almost always where to start. It tells you *which node* and *which
dimension*, which is far more actionable than a dssim score. Numbers that differ
by exactly one line height mean a wrapping difference; numbers off by 1px mean
rounding; wildly different numbers mean a constraint is not reaching the measure
function.

Grid subtrees are compared by box only — the TypeScript dump walks the Yoga
tree, which has no children under a grid because they live in a side cache.

Current: **40 of 56 exact**.

### 2. Raster goldens

```bash
cargo run --release -p sone-goldens --bin sone-goldens [name-filter]
```

Renders each IR document at density 2 and compares with dssim against the
committed JPEG. Writes `target/goldens/report.html` with side-by-side images.

A size mismatch short-circuits the comparison and is reported as an error — that
is a layout problem, so go back to the layout diff.

Current: **43 of 53 under the 0.02 default**, 10 waived.

### 3. Break parity

```bash
cargo test -p sone-skia --test break_parity -- --nocapture
```

Every string in the fixture corpus with its `Intl.Segmenter` breaks, from
`fixtures/visual/break-corpus.json`.

- `uax29_scripts_match_exactly` — hard gate, all 1937 non-dictionary strings
- `dictionary_scripts_hold_their_ratchet` — Khmer et al., ratcheted at 33%

### 4. Measure parity

```bash
cargo test -p sone-skia --test measure_parity
```

Glyph advance widths and font metrics against values dumped from CanvasKit,
to 0.05px. Small, but it catches font-configuration drift before it turns into
dozens of moved goldens.

---

## Diagnosing a moved golden

1. **Run the layout diff for that fixture.** If layout moved, it is a layout or
   text-measurement problem, not a drawing one.
2. **If layout is clean but pixels moved**, it is drawing. Open
   `target/goldens/report.html` and look — a missing dash pattern, a wrong fill
   rule and an off-by-a-pixel shadow all look completely different.
3. **If a whole class of fixtures moved at once**, suspect measurement, not
   layout. Add the offending string to `measure_parity.rs` with the width from
   the TypeScript engine and see whether it disagrees.
4. **Compare renders directly** when the report is not enough:

   ```bash
   cargo run --release -p sone-goldens --bin render-all target/renders <filter>
   ```

   which also writes `index.html` with the goldens side by side.

To get a number out of the TypeScript engine for a specific string, add a case
to its `test/visual/dump-breaks.ts` or write a scratch script against
`renderer.measureText` in a checkout of `seanghay/sone`.

---

## Waivers

`goldens-waivers.toml`. Every entry needs a written reason; `threshold` raises
the ceiling for one fixture, `skip` drops it. A waiver also covers a canvas-size
mismatch, so a documented divergence that changes dimensions does not read as an
error.

Waivers are for **understood** divergences. If you cannot explain it, it is not
a waiver, it is a bug.

The four open ones:

1. **Khmer line breaking** — Skia's ICU dictionary differs from V8's.
   36/108 Khmer strings agree. Affects `text-3`, `text-layout-1`,
   `text-orientation`.
2. **Table cell cross-sizing** — taffy derives a flex item's hypothetical cross
   size from its natural content width, so a cell whose text is wider than its
   column wraps before the column minimum widens it; Yoga re-measures at the
   final width. Affects `table-2`, `table-span-row`, `text-1`, and shows up as
   mid-word splits in `doc.pdf`'s extracted text. Setting `size.width` or
   `flex_basis` on the cell was tried and made every table far worse.
3. **Flex free-space distribution** with `textWrap: "balance"` — Yoga splits a
   row's free space 526/365 between two `flex(1)` columns where taffy splits it
   evenly. The balanced paragraph itself is identical.
4. **Remote assets** — never fetched during a render; `text-2` is skipped.

---

## Refreshing the corpus

```bash
tools/sync-fixtures.sh ../sone
```

Runs the dumpers in a `seanghay/sone` checkout and copies IR documents, layout
trees, the break corpus, goldens, fonts and images across.

The TypeScript engine is the reference and the flow is one-way. If a golden
changes, the question is always "did TypeScript change, or did we?" — check
`git log` in that repo before assuming the Rust side regressed.

---

## Adding a fixture

Add a visual script to `seanghay/sone` under `test/visual/`, using
`writeCanvasToFile` (or `writePdfToFile` for a paginated one) so the dumpers pick
it up, then run `tools/sync-fixtures.sh`. It joins all four harnesses
automatically — no registration anywhere.
