<?php

declare(strict_types=1);

/**
 * A plain-PHP runner rather than PHPUnit, so the suite works from a checkout
 * with no composer install.
 *
 *     php tests/run.php
 */
require_once __DIR__ . '/../src/autoload.php';

use function Sone\{BulletList, Column, ListItem, PageBreak, Photo, Row, Span, Table, TableCell, TableRow, Text};
use Sone\{AssetException, Engine, FontSource, Granularity, IrException, JustifyContent,
    LastPageHeight, Margin, Node, PageBreakMode, ScaleType, Sone, TextAlign};

final class Runner
{
    private int $passed = 0;
    /** @var list<string> */
    private array $failures = [];
    private string $current = '';

    public function test(string $name, \Closure $body): void
    {
        $this->current = $name;
        try {
            $body($this);
            $this->passed++;
            echo '.';
        } catch (\Throwable $e) {
            $this->failures[] = "$name\n    " . $e->getMessage();
            echo 'F';
        }
    }

    public function assert(bool $condition, string $message = 'assertion failed'): void
    {
        if (!$condition) {
            throw new \RuntimeException($message);
        }
    }

    public function equals(mixed $expected, mixed $actual, string $message = ''): void
    {
        if ($expected !== $actual) {
            throw new \RuntimeException(trim($message . ' expected ' . json_encode($expected)
                . ', got ' . json_encode($actual)));
        }
    }

    public function throws(string $class, \Closure $body): \Throwable
    {
        try {
            $body();
        } catch (\Throwable $e) {
            if (!$e instanceof $class) {
                throw new \RuntimeException("expected $class, got " . $e::class . ': ' . $e->getMessage());
            }

            return $e;
        }

        throw new \RuntimeException("expected $class, nothing was thrown");
    }

    public function report(): int
    {
        $total = $this->passed + \count($this->failures);
        echo "\n\n";
        foreach ($this->failures as $failure) {
            echo "FAIL: $failure\n\n";
        }
        echo "$total runs, {$this->passed} passed, " . \count($this->failures) . " failed\n";

        return $this->failures === [] ? 0 : 1;
    }
}

$root = \Sone\Native::checkoutRoot();
$fontPath = "$root/fixtures/font/GeistMono-Regular.ttf";
$family = 'Geist Mono';

$props = static fn (Node $node): array => $node->toIr()['props'] ?? [];

$t = new Runner();

// ── the builder, which touches no native code ───────────────────────────────

$t->test('fluent chaining keeps the concrete type', function (Runner $t) use ($props) {
    $node = Column()->gap(20)->padding(20)->bg('khaki')->rounded(8);
    $t->equals('Sone\ColumnNode', $node::class);
    $t->equals(20, $props($node)['padding']);
});

$t->test('dims take numbers, percentages and auto', function (Runner $t) use ($props) {
    $p = $props(Column()->width(100)->minWidth('50%')->maxWidth('auto'));
    $t->equals(100, $p['width']);
    $t->equals('50%', $p['minWidth']);
    $t->equals('auto', $p['maxWidth']);
});

$t->test('a bad dim is rejected at the call site', function (Runner $t) {
    $t->throws(InvalidArgumentException::class, static fn () => Column()->width('wide'));
});

$t->test('size with one argument is a square', function (Runner $t) use ($props) {
    $p = $props(Column()->size(50));
    $t->equals(50, $p['width']);
    $t->equals(50, $p['height']);
});

$t->test('box shorthand follows CSS', function (Runner $t) use ($props) {
    $p = $props(Column()->padding(10, 20));
    $t->equals([10, 20, 10, 20], [$p['paddingTop'], $p['paddingRight'], $p['paddingBottom'], $p['paddingLeft']]);
    $t->assert(!isset($p['padding']));
});

$t->test('named arguments fill the missing sides the CSS way', function (Runner $t) use ($props) {
    $p = $props(Column()->padding(top: 8, left: 4));
    $t->equals([8, 8, 8, 4], [$p['paddingTop'], $p['paddingRight'], $p['paddingBottom'], $p['paddingLeft']]);
});

$t->test('one value uses the shorthand property', function (Runner $t) use ($props) {
    $t->equals(12, $props(Column()->margin(12))['margin']);
});

$t->test('backed enums and strings are both accepted', function (Runner $t) use ($props) {
    $p = $props(Row()->justifyContent(JustifyContent::SpaceBetween)->alignItems('center'));
    $t->equals('space-between', $p['justifyContent']);
    $t->equals('center', $p['alignItems']);
});

$t->test('background layers accumulate and take a photo', function (Runner $t) use ($props) {
    $layers = $props(Column()->bg('red')->bg(Photo('wall.png')))['background'];
    $t->equals('red', $layers[0]);
    $t->equals('photo', $layers[1]->toIr()['type']);
});

$t->test('filters keep the order they were added in', function (Runner $t) use ($props) {
    $t->equals(['blur(4px)', 'grayscale(0.5)'], $props(Column()->blur(4)->grayscale(0.5))['filters']);
});

$t->test('Text::size is the font size, not the box size', function (Runner $t) use ($props) {
    // The trait conflict is resolved with `insteadof`, which is what encodes this.
    $p = $props(Text('Hello')->size(28));
    $t->equals(28, $p['size']);
    $t->assert(!isset($p['width']));
});

$t->test('the box size is still reachable under another name', function (Runner $t) use ($props) {
    $t->equals(120, $props(Text('Hello')->boxSize(120))['width']);
});

$t->test('text takes strings and spans', function (Runner $t) {
    $inline = Text('Hello ', Span('world')->weight('bold'))->toIr()['inline'];
    $t->equals('Hello ', $inline[0]);
    $t->equals('span', $inline[1]['type']);
    $t->equals('bold', $inline[1]['props']['weight']);
});

$t->test('a decoration colour can be explicitly null', function (Runner $t) use ($props) {
    $p = $props(Text('x')->underline()->underlineColor());
    $t->assert(array_key_exists('underlineColor', $p));
    $t->equals(null, $p['underlineColor']);
});

$t->test('null children are dropped', function (Runner $t) {
    $show = false;
    $ir = Column(Column(), $show ? Row() : null)->toIr();
    $t->equals(1, \count($ir['children']));
});

$t->test('the spread operator generates children', function (Runner $t) {
    $rows = [['a', 'b'], ['c', 'd']];
    $table = Table(...array_map(
        static fn (array $cells): Node => TableRow(...array_map(
            static fn (string $cell): Node => TableCell(Text($cell)),
            $cells,
        )),
        $rows,
    ));
    $t->equals(2, \count($table->toIr()['children']));
    $t->equals('a', $table->toIr()['children'][0]['children'][0]['children'][0]['inline'][0]);
});

$t->test('lists are BulletList because list is reserved', function (Runner $t) use ($props) {
    $list = BulletList(ListItem(Text('one')))->listStyle('disc')->markerGap(8);
    $t->equals('disc', $props($list)['listStyle']);
    $t->equals(1, \count($list->toIr()['children']));
});

$t->test('page break factory and property', function (Runner $t) use ($props) {
    $t->equals('before', $props(PageBreak())['pageBreak']);
    $t->equals('avoid', $props(Column()->pageBreak(PageBreakMode::Avoid))['pageBreak']);
});

$t->test('photo scale type takes a keyword alignment', function (Runner $t) use ($props) {
    $p = $props(Photo('a.png')->scaleType(ScaleType::Cover, 'center'));
    $t->equals('cover', $p['scaleType']);
    $t->equals(0.5, $p['scaleAlignment']);
});

$t->test('the document carries the schema version', function (Runner $t) {
    $document = Sone::render(Column())->toIr();
    $t->equals(1, $document['sone']);
    $t->assert(!isset($document['config']));
});

$t->test('config is written when set', function (Runner $t) {
    $config = Sone::render(Column(), width: 420, pageHeight: 800, margin: new Margin(top: 20))->toIr()['config'];
    $t->equals(420, $config['width']);
    $t->equals(20, $config['margin']['top']);
});

$t->test('pagination tokens are passed through untouched', function (Runner $t) {
    $json = Sone::render(Column(), pageHeight: 800, header: Text('Page {pageNumber}'))->toJson();
    $t->assert(str_contains($json, '{pageNumber}'));
});

$t->test('non-ascii text survives unescaped', function (Runner $t) {
    $t->assert(str_contains(Sone::render(Text('អក្សរ'))->toJson(), 'អក្សរ'));
});

// ── everything that crosses the C ABI ───────────────────────────────────────

$engine = new Engine($root);
$engine->registerFontFile($family, $fontPath);

$t->test('renders a png', function (Runner $t) use ($engine) {
    $png = Sone::render(Column()->size(16)->bg('red'), engine: $engine)->png();
    $t->equals("\x89PNG", substr($png, 0, 4));
});

$t->test('density scales the raster', function (Runner $t) use ($engine) {
    $node = static fn (): Node => Column()->size(10)->bg('red');
    // Raw is 4 bytes per pixel, so the byte count is the pixel count.
    $t->equals(10 * 10 * 4, \strlen(Sone::render($node(), engine: $engine)->raw()));
    $t->equals(20 * 20 * 4, \strlen(Sone::render($node(), engine: $engine)->raw(2.0)));
});

$t->test('renders every format', function (Runner $t) use ($engine) {
    $r = Sone::render(Column()->size(16)->bg('teal'), engine: $engine);
    $t->assert($r->jpeg(0.8) !== '');
    $t->assert($r->webp() !== '');
    $t->equals('%PDF', substr($r->pdf(), 0, 4));
    $t->assert(str_contains($r->svg(), '<svg'));
});

$t->test('one page per declared break', function (Runner $t) use ($engine) {
    $root = Column(
        Column()->height(60)->bg('red'),
        Column()->height(60)->bg('green')->pageBreak(PageBreakMode::Before),
        Column()->height(60)->bg('blue')->pageBreak(PageBreakMode::Before),
    );
    $pages = Sone::render($root, engine: $engine, width: 40, pageHeight: 200)->pages();
    $t->equals(3, \count($pages));
    foreach ($pages as $page) {
        $t->equals("\x89PNG", substr($page, 0, 4));
    }
});

$t->test('the font registry round trips', function (Runner $t) use ($root, $family, $fontPath) {
    $fresh = new Engine($root);
    $t->assert(!$fresh->hasFont($family));
    $fresh->registerFontFile($family, $fontPath);
    $t->assert($fresh->hasFont($family));
    $t->assert(\in_array($family, $fresh->fontFamilies(), true));
    $fresh->resetFonts();
    $t->assert(!$fresh->hasFont($family));
    $fresh->registerFont($family, file_get_contents($fontPath));
    $t->assert($fresh->hasFont($family));
    $fresh->close();
});

$t->test('registered images resolve as assets', function (Runner $t) use ($engine) {
    $png = Sone::render(Column()->size(8)->bg('red'), engine: $engine)->png();
    $engine->registerImage('logo', $png);
    $t->assert(Sone::render(Photo('asset:logo')->size(8), engine: $engine)->png() !== '');
});

$t->test('layout comes back as a tree', function (Runner $t) use ($engine) {
    $layout = Sone::render(Column(Column()->size(20)->tag('inner'))->padding(5), engine: $engine)->layout();
    // The layout dump is JSON, so every number arrives as a float.
    $t->equals(30.0, $layout['width']);
    $t->equals('inner', $layout['children'][0]['tag']);
});

$t->test('metadata honours granularity', function (Runner $t) use ($engine, $family) {
    $r = Sone::render(Text('hello world')->font($family)->size(12), engine: $engine);
    $t->assert(\is_array($r->metadata()));
    $t->assert(\is_array($r->metadata(Granularity::Word)));
});

$t->test('a bad document is an IR error', function (Runner $t) use ($engine) {
    $e = $t->throws(IrException::class, static fn () => $engine->render('{"sone":99,"root":{"type":"column"}}'));
    $t->assert(str_contains($e->getMessage(), 'unsupported IR version'));
});

$t->test('a missing font file is an asset error', function (Runner $t) use ($engine) {
    $t->throws(AssetException::class, static fn () => $engine->registerFontFile('Nope', 'does/not/exist.ttf'));
});

$t->test('save infers the format from the extension', function (Runner $t) use ($engine) {
    $path = sys_get_temp_dir() . '/sone-php-' . getmypid() . '.pdf';
    Sone::render(Column()->size(16)->bg('red'), engine: $engine)->save($path);
    $t->equals('%PDF', substr(file_get_contents($path), 0, 4));
    unlink($path);
});

// The gate every binding owes: the same document must come out of this binding
// byte for byte the way it comes out of `sone-cli`.
$t->test('matches the CLI byte for byte', function (Runner $t) use ($engine, $root, $family, $fontPath) {
    $node = Column(
        Text('Hello ', Span('world')->weight('bold')->color('#c0392b'))
            ->font($family)->size(24)->lineHeight(1.4),
        Row(
            Column()->bg('lightgreen')->size(50)->borderRadius(14),
            Column()->bg('salmon')->height(50)->borderRadius(14)->flex(1),
        )->gap(10),
    )->gap(20)->padding(20)->size(420, 200)->bg('khaki')->cornerRadius(28);

    // An absolute src, because the CLI resolves a document's assets against the
    // document's own directory and the engine resolves them against its base
    // directory — the two only agree when the path is absolute.
    $rendering = Sone::render($node, engine: $engine, density: 2, fonts: [new FontSource($family, $fontPath)]);

    $directory = sys_get_temp_dir() . '/sone-parity-' . getmypid();
    @mkdir($directory);
    $document = "$directory/doc.json";
    $fromCli = "$directory/cli.png";
    file_put_contents($document, $rendering->toJson(pretty: true));

    $command = sprintf(
        'cd %s && cargo run -q -p sone-cli -- render %s --density 2 -o %s 2>&1',
        escapeshellarg($root), escapeshellarg($document), escapeshellarg($fromCli),
    );
    exec($command, $output, $status);
    $t->equals(0, $status, implode("\n", $output));
    $t->assert(file_get_contents($fromCli) === $rendering->png(), 'bytes differ from the CLI');

    array_map('unlink', glob("$directory/*"));
    rmdir($directory);
});

$engine->close();
exit($t->report());
