# sone for Ruby

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

```ruby
require "sone"

Sone::Font.load("Inter", "fonts/Inter-Regular.ttf")

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

Sone.render(root, density: 2).save("card.png")
```

## The shape of the API

**A block DSL, not a fluent chain.** Setting a property and adding a child are
the same kind of call — `gap 20` sets, `row do … end` appends — so a block reads
top to bottom as "what this box is, then what is in it". `do…end` for
multi-line, `{ … }` with semicolons for a one-liner.

Nothing is defined on `Object`. There is no `include Sone::DSL` and nothing to
collide with; the names only exist inside a block.

**A no-argument call reads.** `gap` returns what is set, `gap 20` sets it, so a
block can branch on what it has already been given:

```ruby
Sone.column do
  gap 8
  padding gap * 2
end
```

The exceptions are flags and decorations — `nowrap`, `autofit`, `underline`,
`overline`, `line_through`, `flip_horizontal` — which turn *on* with a bare
call, because a bare call that silently did nothing would be a trap.

**Symbols are keywords.** `:space_between` becomes `"space-between"`, so the
whole kebab-case vocabulary is reachable without quotes. Plain strings still
work everywhere.

**Generated children are just Ruby.** This is what the block form buys over
fluent chaining, where the same thing needs a splat or a `map`:

```ruby
Sone.table do
  spacing 0, 8
  records.each do |record|
    table_row do
      record.cells.each { |cell| table_cell { text cell } }
    end
  end
  table_row { table_cell { text "No results" } } if records.empty?
end
```

**Text reads as text.**

```ruby
Sone.text("Hello ") do
  font "Inter"
  size 28
  line_height 1.4
  align :justify

  span("world") { weight :bold; color "salmon" }
end
```

## The three things to know

**`instance_eval` moves `self`.** Inside a bare block, `@ivars` and helper
methods on the enclosing object stop resolving — while local variables still
close over fine, which is exactly what makes it confusing. Take a block
argument and `self` stays yours:

```ruby
Sone.column do |c|      # self is still your object
  c.bg @colour
  c.size 50
end
```

Both forms work, decided per block by the block's arity.

**Locals shadow the DSL.** A `size = 50` earlier in the same scope turns a later
`size 50` into a parse error, because Ruby resolves the bare name to the local.
Nothing the library can do about it. `self.size 50` is the escape hatch.

**`end` is a Ruby keyword**, so the trailing inset is `inset_end`. The keyword
spelling works with an explicit receiver: `self.end 8`.

Every property is also available in camelCase (`cornerRadius`, `alignItems`), so
an example from the TypeScript engine transfers with no edits.

## Engine and output

```ruby
engine = Sone::Engine.new("assets")
engine.register_font("Inter", File.binread("Inter-Regular.ttf"))
engine.register_image("logo", png_bytes)          # reachable as asset:logo

rendering = Sone.render(root, engine: engine, width: 816, page_height: 1056,
                        header: Sone.text("Page {pageNumber} of {totalPages}"))

rendering.pdf                                     # selectable text, one page per break
rendering.pages                                   # one raster image per page
rendering.save("report.pdf")
rendering.save_pages("page.png")                  # page-1.png, page-2.png, ...
rendering.layout                                  # the computed layout tree
rendering.metadata(:word)                         # dataset-style boxes
rendering.to_json(pretty: true)                   # the IR itself
engine.close
```

Skia carries no system fonts, so at least one family must be registered before
any text renders. `Sone::Font.load` does it on the process-wide engine for
scripts that do not want to own one.

Header and footer text uses the literal tokens `{pageNumber}` and
`{totalPages}`; the engine substitutes them during pagination.

**One engine per thread.** Skia's font collection is shared inside an engine, so
every call takes a mutex. Give each thread its own engine for real parallelism
rather than sharing one.

Failures arrive as `Sone::IrError`, `Sone::AssetError` or `Sone::RenderError`,
all under `Sone::Error`.

## Installing

The gem is pure Ruby over the C ABI in `include/sone.h` using the `ffi` gem, so
there is no Rust toolchain and no build step. What it does need is the native
library.

**The native library is not in the gem yet.** For now, build it from a checkout:

```bash
cargo build --release -p sone-ffi        # produces target/release/libsone.dylib
```

The binding finds it by walking up to the checkout root. Anywhere else, point
`SONE_NATIVE_LIBRARY` at the file or at the directory holding it.

## Development

```bash
cd bindings/ruby
bundle install
rake test
```

The suite includes the parity gate every sone binding owes: the same document
rendered through this binding and through `sone-cli` must come out byte for byte
identical.

## Design

Every sone binding is thin. The DSL is reimplemented per language and produces
the same JSON **IR document**; the native layer is document-in, bytes-out.
Layout, text and drawing exist exactly once, in Rust.

So the interesting part of this gem is pure Ruby: the tree is built and
serialized with no native code involved, and `to_json` works before an engine
exists.
