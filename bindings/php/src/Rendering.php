<?php

declare(strict_types=1);

namespace Sone;

/** Page margins. A single number applies to all four sides. */
final class Margin implements \JsonSerializable
{
    public function __construct(
        public readonly int|float $top = 0,
        public readonly int|float $right = 0,
        public readonly int|float $bottom = 0,
        public readonly int|float $left = 0,
    ) {}

    public static function all(int|float $value): self
    {
        return new self($value, $value, $value, $value);
    }

    public function jsonSerialize(): array
    {
        return ['top' => $this->top, 'right' => $this->right, 'bottom' => $this->bottom, 'left' => $this->left];
    }
}

/** A font the document carries with it, so another sone engine renders it identically. */
final class FontSource
{
    public function __construct(public readonly string $name, public readonly string $src) {}
}

/** A node plus its render configuration, with one method per output format. */
final class Rendering
{
    private const FORMAT_BY_EXTENSION = [
        'png' => OutputFormat::Png, 'jpg' => OutputFormat::Jpeg, 'jpeg' => OutputFormat::Jpeg,
        'webp' => OutputFormat::Webp, 'pdf' => OutputFormat::Pdf, 'svg' => OutputFormat::Svg,
        'raw' => OutputFormat::Raw, 'rgba' => OutputFormat::Raw,
    ];

    private ?string $cached = null;

    /** @param list<FontSource> $fonts */
    public function __construct(
        private readonly Node $root,
        private readonly ?Engine $engine = null,
        private readonly int|float|null $width = null,
        private readonly int|float|null $height = null,
        private readonly ?string $background = null,
        private readonly int|float|null $density = null,
        private readonly int|float|null $pageHeight = null,
        private readonly Margin|int|float|null $margin = null,
        private readonly LastPageHeight|string|null $lastPageHeight = null,
        private readonly ?Node $header = null,
        private readonly ?Node $footer = null,
        private readonly array $fonts = [],
    ) {}

    public function engine(): Engine
    {
        return $this->engine ?? Engine::default();
    }

    // ── the document ────────────────────────────────────────────────────────

    /** The IR document as an array. */
    public function toIr(): array
    {
        $config = array_filter([
            'width' => $this->width,
            'height' => $this->height,
            'background' => $this->background,
            'density' => $this->density,
            'pageHeight' => $this->pageHeight,
            'margin' => \is_object($this->margin) ? $this->margin->jsonSerialize() : $this->margin,
            'lastPageHeight' => $this->lastPageHeight instanceof LastPageHeight
                ? $this->lastPageHeight->value
                : $this->lastPageHeight,
            'header' => $this->header?->toIr(),
            'footer' => $this->footer?->toIr(),
        ], static fn (mixed $value): bool => $value !== null);

        $document = ['sone' => 1];
        if ($this->fonts !== []) {
            $document['fonts'] = array_map(
                static fn (FontSource $font): array => ['name' => $font->name, 'src' => $font->src],
                $this->fonts,
            );
        }
        if ($config !== []) {
            $document['config'] = $config;
        }
        $document['root'] = $this->root->toIr();

        return $document;
    }

    /** The IR document as JSON, built once and reused. */
    public function toJson(bool $pretty = false): string
    {
        $flags = JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES;
        if ($pretty) {
            return json_encode($this->toIr(), $flags | JSON_PRETTY_PRINT);
        }

        return $this->cached ??= json_encode($this->toIr(), $flags);
    }

    // ── outputs ─────────────────────────────────────────────────────────────

    public function png(?float $density = null): string
    {
        return $this->engine()->render($this->toJson(), OutputFormat::Png, $density);
    }

    public function jpeg(float $quality = 1.0, ?float $density = null): string
    {
        return $this->engine()->render($this->toJson(), OutputFormat::Jpeg, $density, $quality);
    }

    public function webp(float $quality = 1.0, ?float $density = null): string
    {
        return $this->engine()->render($this->toJson(), OutputFormat::Webp, $density, $quality);
    }

    /** Raw RGBA pixels, row-major, unpremultiplied. */
    public function raw(?float $density = null): string
    {
        return $this->engine()->render($this->toJson(), OutputFormat::Raw, $density);
    }

    /** A PDF. With `pageHeight` set, one page per break and selectable text. */
    public function pdf(): string
    {
        return $this->engine()->render($this->toJson(), OutputFormat::Pdf);
    }

    public function svg(): string
    {
        return $this->engine()->render($this->toJson(), OutputFormat::Svg);
    }

    /** One raster image per page. Requires `pageHeight`. @return list<string> */
    public function pages(OutputFormat|string $format = OutputFormat::Png, ?float $density = null, float $quality = 1.0): array
    {
        return $this->engine()->renderPages($this->toJson(), $format, $density, $quality);
    }

    /** Render and write to `$path`, inferring the format from its extension. */
    public function save(string $path, ?float $density = null, float $quality = 1.0): string
    {
        $bytes = match ($this->formatFor($path)) {
            OutputFormat::Png => $this->png($density),
            OutputFormat::Jpeg => $this->jpeg($quality, $density),
            OutputFormat::Webp => $this->webp($quality, $density),
            OutputFormat::Raw => $this->raw($density),
            OutputFormat::Pdf => $this->pdf(),
            OutputFormat::Svg => $this->svg(),
        };
        file_put_contents($path, $bytes);

        return $path;
    }

    /** Write `name-1.png`, `name-2.png`, … next to `$path`. @return list<string> */
    public function savePages(string $path, ?float $density = null, float $quality = 1.0): array
    {
        $extension = pathinfo($path, PATHINFO_EXTENSION);
        $stem = $extension === '' ? $path : substr($path, 0, -\strlen($extension) - 1);
        $suffix = $extension === '' ? '.png' : ".$extension";
        $format = self::FORMAT_BY_EXTENSION[strtolower($extension)] ?? OutputFormat::Png;

        $written = [];
        foreach ($this->pages($format, $density, $quality) as $index => $bytes) {
            $name = $stem . '-' . ($index + 1) . $suffix;
            file_put_contents($name, $bytes);
            $written[] = $name;
        }

        return $written;
    }

    // ── introspection ───────────────────────────────────────────────────────

    /** The computed layout tree. */
    public function layout(): array
    {
        return json_decode($this->engine()->dumpLayout($this->toJson()), true);
    }

    /** Dataset-style boxes at node, line or word granularity. */
    public function metadata(Granularity|string $granularity = Granularity::Node): array
    {
        return json_decode($this->engine()->dumpMetadata($this->toJson(), $granularity), true);
    }

    private function formatFor(string $path): OutputFormat
    {
        $extension = strtolower(pathinfo($path, PATHINFO_EXTENSION));

        return self::FORMAT_BY_EXTENSION[$extension]
            ?? throw new \InvalidArgumentException("cannot infer an output format from \"$path\"");
    }
}

/**
 * Wrap a node with render configuration.
 *
 *     Sone::render($root, density: 2)->save('card.png');
 */
final class Sone
{
    /** @param list<FontSource> $fonts */
    public static function render(
        Node $root,
        ?Engine $engine = null,
        int|float|null $width = null,
        int|float|null $height = null,
        ?string $background = null,
        int|float|null $density = null,
        int|float|null $pageHeight = null,
        Margin|int|float|null $margin = null,
        LastPageHeight|string|null $lastPageHeight = null,
        ?Node $header = null,
        ?Node $footer = null,
        array $fonts = [],
    ): Rendering {
        return new Rendering($root, $engine, $width, $height, $background, $density,
            $pageHeight, $margin, $lastPageHeight, $header, $footer, $fonts);
    }
}
