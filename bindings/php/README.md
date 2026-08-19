# sone for PHP

A declarative canvas layout engine with rich international text — flexbox
layout, Skia rendering, and PNG / JPEG / WebP / PDF / SVG output.

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

Sone::render($root, density: 2)->save('card.png');
```

## The shape of the API

**Every setter returns `static`,** so a chain keeps the concrete type and PHP's
named arguments work throughout:

```php
->padding(top: 20, left: 4)          // omitted sides follow CSS
->scaleType(ScaleType::Cover, alignment: 'center')
```

**Backed enums, and strings too.** `JustifyContent::SpaceBetween` gets you
completion; `'space-between'` still compiles, so a value the engine understands
is never unreachable.

**Spread for generated children:**

```php
Table(...array_map(
    fn (array $cells) => TableRow(...array_map(
        fn (string $cell) => TableCell(Text($cell)),
        $cells,
    )),
    $rows,
));
```

A null child is dropped, so `$flag ? Badge() : null` needs no filtering.

**Traits, and `insteadof` where they collide.** `Text` uses all three property
traits, and `LayoutProps::size()` (the box) and `SpanStyleProps::size()` (the
font) collide on the name. PHP's `insteadof` resolves it exactly:

```php
use LayoutProps, SpanStyleProps, TextBlockProps {
    SpanStyleProps::size insteadof LayoutProps;
    LayoutProps::size as boxSize;
}
```

So `Text('Hi')->size(28)` is the font size, matching the TypeScript API, and the
box size is still reachable as `boxSize()`.

## Two names that had to move

`list` is a reserved word in PHP, so the list factory is **`BulletList()`** and
the class is `ListNode`. Everything else keeps its TypeScript spelling.

## Engine and output

```php
$engine = new Engine('assets');
$engine->registerFont('Inter', file_get_contents('Inter-Regular.ttf'));
$engine->registerImage('logo', $pngBytes);        // reachable as asset:logo

$rendering = Sone::render($root, engine: $engine, width: 816, pageHeight: 1056,
    header: Text('Page {pageNumber} of {totalPages}'));

$rendering->pdf();                                 // selectable text, one page per break
$rendering->pages();                               // one raster image per page
$rendering->save('report.pdf');
$rendering->savePages('page.png');                 // page-1.png, page-2.png, ...
$rendering->layout();                              // the computed layout tree
$rendering->metadata(Granularity::Word);
$rendering->toJson(pretty: true);                  // the IR itself
$engine->close();
```

Skia carries no system fonts, so at least one family must be registered before
any text renders. Header and footer text uses the literal tokens
`{pageNumber}` and `{totalPages}`; the engine substitutes them.

Failures arrive as `Sone\IrException`, `Sone\AssetException` or
`Sone\RenderException`, all under `Sone\SoneException`.

## Installing

FFI over the C ABI in `include/sone.h`, so `composer require` needs no build
step and no extension to compile. What it does need:

- **`ext-ffi`.** Enabled by default in CLI. In a typical production config it is
  gated behind `ffi.enable=preload`, which means the library has to be loaded
  from a preload script rather than per-request.
- **The native library.** Not in the package yet — build it from a checkout with
  `cargo build --release -p sone-ffi`. The binding finds it by walking up to the
  checkout root; anywhere else, set `SONE_NATIVE_LIBRARY`.

`sone.h` next to `src/` is a stripped copy of the canonical header: PHP's FFI
parser is not a C preprocessor, so it cannot read `#include` or the
`extern "C"` guards. Regenerate it with `php tools/generate-header.php` whenever
the C ABI changes.

## Development

```bash
cd bindings/php
php tests/run.php
```

A plain-PHP runner rather than PHPUnit, so the suite works with no composer
install. It includes the parity gate every sone binding owes: the same document
rendered through this binding and through `sone-cli` must come out byte for byte
identical.
