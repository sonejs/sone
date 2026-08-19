# Porting notes

Everything in the engine that looks strange but is deliberate. Each entry is a
behaviour that cost real time to find, with the symptom that led to it, so a
future change that reintroduces the bug is recognisable.

If you change layout or text code and a golden moves, read this first — there is
a decent chance the answer is already here.

---

## Text measurement

### Variable fonts must pin every axis

`sone-skia/src/text.rs` builds a `VariationPosition` containing **every** axis
the primary family exposes, not just `wght`.

CanvasKit leaves unspecified axes at their default value. Bare Skia clamps them
to the axis *minimum*. Google Sans has `opsz` with range 17–18 and default 18, so
specifying only `wght` silently pinned `opsz` to 17 and every string measured
about 2% wide.

The symptom was subtle and global: text fitted one word fewer per line across
every fixture that used Google Sans, so half the corpus was off by a line.
`sone-skia/tests/measure_parity.rs` holds widths dumped from the TypeScript
engine and would have caught it immediately — write the parity test first next
time.

### The measure contract is inside-out

taffy's measure callback receives:

- `known_dimensions` — the **outer** (border-box) size, *not* reduced by padding
- `available_space` — the **inner** size, already reduced by padding, border and
  margin, and already clamped by min/max

That is the opposite of the natural assumption. Wrapping must read
`available_space`; `known_dimensions` is only useful for detecting "this axis is
already decided".

Getting this backwards made text wrap at the border-box width, so a
`width(72).padding(14)` box wrapped at 100px instead of 72px and came out two
lines short. See `constrain()` and `inner()` in `layout/engine.rs`.

### Yoga folds max-size into the constraint; taffy does not

taffy only clamps a *definite* available space. When the probe is
`MaxContent`, a node's own `maxWidth` is never applied, so text lays out
unwrapped and the intrinsic size explodes.

`constrain()` reapplies it. Without this, `text-wrap-balance-1` measured 2280px
wide against a golden of 1982.

### Rotated text wraps against a real constraint only

For `orientation: 90 | 270` the wrap width comes from the node's own `height` or
`maxHeight` — never from the available cross-axis space.

taffy asks "how tall are you if the available height is 62.4?" while it is still
*deciding* the row's cross size. Feeding that back into wrapping made "EAST"
wrap into two lines and the pillar came out 114×86 instead of 80×102. Yoga never
makes that query, because it derives the cross size from the item's own
unconstrained measurement.

### min-content probes

`available_space == MinContent` means "how narrow can you get". Wrapping at zero
returns the widest unbreakable piece. Without handling this, a min-content probe
returned the *max*-content width, and taffy's automatic minimum sizing then
refused to shrink flex items — a list item's content column came out 777px wide
inside a 416px list.

Do not "fix" that by forcing `min_size: 0` to imitate Yoga's lack of automatic
minimum sizing. That was tried; it made 48 of 56 layouts worse.

---

## Layout

### Pixel rounding is Yoga's, not taffy's

taffy's own rounding is switched off (`tree.disable_rounding()`), and
`round_value()` in `layout/engine.rs` reimplements Yoga's
`roundValueToPixelGrid`, including the rule that nodes with a measure function
(text and grid) round **outward** — `ceil` on the far edge when the size has a
fractional part, `floor` on the near edge.

This one change took the layout diff from 52 failing fixtures to 26. Before it,
text boxes were consistently 1px short (`ts=17 rust=16`) because taffy rounds to
nearest while Yoga refuses to round a measured box down into its own content.

Rounding walks *unrounded* absolute positions down the tree and only rounds at
each node, exactly as Yoga does — do not accumulate rounded values.

### A layout root is sized exactly, not at most

When `calculateLayout(ownerWidth, ...)` gets a width and the root has no style
width or max-width, Yoga treats it as `Exactly`, not `AtMost`. taffy would size
it to content. `layout_subtree()` forces the root size to match.

### Yoga's defaults, shimmed

`layout/style.rs` starts from Yoga's defaults rather than taffy's:
`flex-direction: column`, `flex-shrink: 0`, `align-content: flex-start`. The
`flex` shorthand also follows Yoga: `grow = max(v, 0)`, `shrink = v < 0 ? -v : 0`,
`basis = v > 0 ? 0 : auto`.

Edge precedence follows Yoga too: `start`/`end` beat `left`/`right`, which beat
the axis shorthand, which beats `all`.

### Measure is skipped when both axes are exact

Yoga does not call the measure function at all when width and height are both
`Exactly`. Nothing depends on this today — taffy passes `Size::NONE` on the
final layout pass, so it never triggers — but the reasoning is recorded because
it looked like the fix for the rotated-text bug and is not.

### Grid and table are hand-rolled

taffy's grid is compiled out entirely (`default-features = false`). The engine
ports the TypeScript `resolveGridLayout` and the four-pass table sizing instead,
because the TypeScript engine hand-rolled both on top of Yoga and matching CSS
grid semantics would be a behaviour change, not a port.

Grid children are laid out in **separate taffy trees**, mirroring how the
TypeScript engine creates detached Yoga nodes. That is why `layout_subtree()` is
the only tree-building entry point and everything recurses through it.

---

## Drawing

### Odd-length dash arrays

Canvas2D duplicates an odd-length dash array; Skia requires an even one and
returns `None` otherwise. `compile.rs` duplicates it. Symptom: `strokeDashArray(14)`
drew a solid line, dssim 0.13 on `path-1`.

### Borders

A uniform border is a double-width stroke inside an antialiased clip of the same
path — that is how the rounded-corner inner stroke is produced. Per-side borders
fall back to four lines with square caps. Both are ports; do not "simplify" them.

### Group opacity opens a layer

`opacity < 1` or any CSS filter opens a `saveLayer`, so overlapping children
composite once rather than double-darkening. A layer is expensive on a software
rasterizer, so it is only opened when actually required.

### Draw order

Shadow → clip → background → border, then children, then (for tables) the grid
lines. `crates/sone-core/tests/draw_order.rs` asserts the sequence against a
recording painter; it is cheap insurance and does not need Skia.

---

## Text engine

### Segmentation is split by script

`sone-skia` uses UAX#29 (`unicode-segmentation`) for everything except Khmer,
Thai, Lao and Burmese, which have no UAX#29 word boundaries at all and go
through Skia's bundled ICU via `Paragraph::get_word_boundary`.

UAX#29 matches `Intl.Segmenter` **exactly** on all 1937 non-dictionary strings.
Skia's ICU alone scored 1918 — it splits at `MidNumLet` dots, so `image.jpg`,
`e.g.` and `Font.load()` broke apart.

ICU4X scored better on Khmer (42/108 versus 36/108) but cost 2.3 MB of duplicate
data on top of the ICU already linked for shaping, so it was removed. If Khmer
line breaking matters more than binary size, that trade is a one-line change back.

Skia reports word boundaries as **UTF-16 offsets**; the engine works in bytes.
`utf16_offsets()` maps between them.

### Measurement is font-derived, never text-derived

Ascent and descent come from `Font::metrics()`, not from a laid-out paragraph, so
line heights do not shift with the glyphs on the line. This is the contract the
TypeScript engine documents at `SoneTextMetrics`, and breaking it produces text
that fits the box it was measured into only sometimes.

### Runs are always laid out unconstrained

Every painted run uses `layout(1e7)` with `TextAlign::Left`, even for RTL. sone
has already broken lines and positioned the segment box; constraining a
paragraph to its own measured width lets floating-point rounding push the last
cluster onto a second line, which then draws stacked inside the segment box and
reads as overlapping text.

---

## Build and packaging

### `embed-icudtl` is a no-op

skia-safe declares the feature; skia-bindings 0.99 defines it as `[]` and never
reads it. `skia_use_icu` is unconditional once `textlayout` is on. Toggling it
produces a byte-identical binary — verified, not assumed. The 10 MB ICU table
cannot be dropped without building Skia from source with `skia_use_libgrapheme`,
which would also lose the Khmer dictionary and the bidi implementation.

### The main workspace aborts on panic; the Python binding unwinds

`panic = "abort"` is correct for a C ABI — unwinding across it is undefined. But
PyO3 needs unwinding to convert panics into Python exceptions, which is why
`bindings/python` is a **separate workspace**.

`sone-ffi` still wraps its entry points in `catch_unwind`, because debug builds
unwind regardless of the release profile.

### PyO3 0.29 renamed `allow_threads`

It is `Python::detach` now. Nothing else in the binding was affected.

---

## Deliberate divergences from the TypeScript engine

These are improvements, each with tests:

- **Radial gradients** are implemented. The TypeScript engine parses
  `radial-gradient` and then skips it.
- **Odd-length dash arrays** behave like Canvas2D.
- CSS strings are parsed **once** at the IR boundary into typed values; nothing
  is re-parsed during layout or draw.
- Errors are `Result`-typed with a JSON pointer for IR failures, mapped onto CLI
  exit codes (2 IR / 3 asset / 4 render) and FFI status codes.

One TypeScript bug was found and *not* reproduced: `compile()` returns early
after configuring a photo background, so a container with a photo background
never compiles its children. No fixture triggers it. The Rust engine compiles
children normally.
