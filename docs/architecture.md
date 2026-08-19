# Architecture

How a document becomes pixels, and why the pieces are split the way they are.

```
IR document (JSON)
  → ir.rs          deserialize into typed props
  → compile.rs     resolve CSS strings, load images, cascade text defaults,
                   expand list markers, apply table spacing
  → layout/        taffy for flexbox; hand-rolled grid and table;
                   text measured through the TextEngine trait
  → draw/          walk the tree against the Painter trait
  → sone-skia      the only place Skia types exist
  → png | jpg | webp | raw | pdf | svg
```

## The crate split is a hard boundary

`sone-core` has **no Skia dependency**. It talks to three traits over plain
value types:

- `Painter` — save/restore, clips, transforms, `draw_rect`, `draw_path`,
  `draw_image`, `draw_text_run`
- `TextEngine` — `measure`, `break_points`, `grapheme_starts`, `has_font`
- `AssetLoader` — bytes for a `src`

Nothing crosses that seam but `PaintSpec`, `LayerSpec`, `BezPath`, `Rect` and
`SpanStyle`. The payoff is concrete: line breakers, justification, pagination
and draw ordering are all unit-testable against
`testing::FixedMetricsEngine` and `testing::RecordingPainter` with no rasterizer
in the process — that is what `crates/sone-core/tests/draw_order.rs` and the
breaker tests do. It also keeps a second backend possible.

## The IR is the contract

Every binding produces the same JSON document; the engine consumes nothing else.
`"sone": 1` versions it. CSS strings stay strings on the wire (colours,
gradients, shadows, filters, SVG path data) and are parsed exactly once at the
boundary — the engine never re-parses a string during layout or draw.

Function-valued props cannot serialize, so IR v1 uses static nodes plus
`{pageNumber}` / `{totalPages}` template tokens, and pre-resolved `marker` nodes
for callback list styles.

## Compile

`compile.rs` mirrors the TypeScript `compile()` step by step, and the order
matters:

- ids are assigned pre-order
- `row` and `table-row` default to `flexDirection: row`, `list` to `column`
- text nodes get `flexShrink: 1` and `boxSizing: content-box`
- spans inherit a specific subset of the block's style — not everything
- `text-default` nodes are flattened away, cascading their style
- list items are rebuilt as `[marker, contentColumn]`
- table `spacing` becomes cell padding

The output is an owned `CompiledNode` tree with typed `BoxStyle`, `RunStyle` and
`BlockStyle` — no clone registry, no id table, no re-parsing.

## Layout

`layout_subtree()` is the only tree-building entry point. Every measurement root
gets a **fresh taffy tree**, mirroring how the TypeScript engine creates
detached Yoga nodes — which is what makes recursive grid measurement possible
without fighting the borrow checker.

Results land in a `BoxLayout` tree plus a `LayoutState` holding paragraphs, grid
resolutions and table grids, keyed by flat node index. Draw and metadata both
read from that, so nothing is measured twice.

Yoga's semantics are reproduced deliberately in several places — defaults,
`flex` shorthand, edge precedence, pixel rounding, root sizing. See
[porting notes](porting-notes.md).

## Text

`text/paragraph.rs` is the port of `text.ts` and the largest single piece:
greedy, Knuth-Plass and balanced breakers; justification by word spacing;
`maxLines` with ellipsis walk-back; tab stops and dot leaders; indents;
per-segment metrics with half-leading line boxes; segment merging.

It only ever calls `TextEngine::measure` and `break_points`, so the whole thing
is testable with fixed metrics.

`draw/text.rs` positions runs (`place_runs`) and is shared by drawing and
metadata, so they cannot drift.

## Draw

A direct port of `drawOnCanvas`, including the ordering rules and the
inner-stroke border trick. `arena.ts` — the manual paint pooling the TypeScript
engine needs for CanvasKit's non-GC objects — has no equivalent here; RAII
covers it.

## Pagination

`pagination.rs` computes break offsets from the laid-out tree: text splits at
line boundaries, `before`/`after`/`avoid` honoured, breaks within 20px collapsed.
`sone-skia/render.rs` turns those into pages with header/footer bands and
per-page token substitution, then into a multi-page PDF or one raster per page.
