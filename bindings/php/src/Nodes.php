<?php

declare(strict_types=1);

namespace Sone;

/** A vertical container. */
final class ColumnNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$children)
    {
        parent::__construct('column');
        $this->adopt($children);
    }
}

/** A horizontal container. */
final class RowNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$children)
    {
        parent::__construct('row');
        $this->adopt($children);
    }
}

/** A grid container with row-major auto placement. */
final class GridNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$children)
    {
        parent::__construct('grid');
        $this->adopt($children);
    }

    public function columns(int|float|string ...$tracks): static { return $this->set('columns', $this->tracks($tracks)); }
    public function rows(int|float|string ...$tracks): static { return $this->set('rows', $this->tracks($tracks)); }
    public function autoRows(int|float|string ...$tracks): static { return $this->set('autoRows', $this->tracks($tracks)); }
    public function autoColumns(int|float|string ...$tracks): static { return $this->set('autoColumns', $this->tracks($tracks)); }

    /** @param list<int|float|string> $tracks @return list<int|float|string> */
    private function tracks(array $tracks): array
    {
        foreach ($tracks as $track) {
            if (\is_string($track) && $track !== 'auto' && !preg_match('/^[\d.]+fr$/', $track)) {
                throw new \InvalidArgumentException("expected a number, \"auto\", or an fr value, got \"$track\"");
            }
        }

        return $tracks;
    }
}

/** A styled run inside a `Text`. */
final class SpanNode extends Node
{
    use SpanStyleProps;

    public function __construct(string $text = '')
    {
        parent::__construct('span');
        if ($text !== '') {
            $this->inline[] = $text;
        }
    }
}

/**
 * A paragraph. Both a box and a run of text.
 *
 * PHP traits collide on `size`, and `insteadof` is exactly the tool for it —
 * which also encodes the rule that `Text::size()` is the font size, matching
 * the TypeScript API where `TextPropsBuilder` omits the layout `size`.
 */
final class TextNode extends Node
{
    use LayoutProps, SpanStyleProps, TextBlockProps {
        SpanStyleProps::size insteadof LayoutProps;
        LayoutProps::size as boxSize;
    }

    public function __construct(string|Node ...$content)
    {
        parent::__construct('text');
        foreach ($content as $item) {
            $this->inline[] = $item;
        }
    }

    /** Whether the paragraph wraps. Not the flexbox `wrap`. */
    public function wrapText(bool $value = true): static
    {
        return $this->set('nowrap', !$value);
    }

    /** Append content after construction. */
    public function content(string|Node ...$items): static
    {
        foreach ($items as $item) {
            $this->inline[] = $item;
        }

        return $this;
    }
}

/** Cascades text styling onto its descendants without drawing a box. */
final class TextDefaultNode extends Node
{
    use SpanStyleProps, TextBlockProps;

    public function __construct(Node|null ...$children)
    {
        parent::__construct('text-default');
        $this->adopt($children);
    }
}

/** An image. */
final class PhotoNode extends Node
{
    use LayoutProps;

    private const ALIGNMENTS = ['start' => 0.0, 'center' => 0.5, 'end' => 1.0];

    public function __construct(string $src)
    {
        parent::__construct('photo');
        $this->set('src', $src);
    }

    /** How the image fills its box. The alignment is 0..1, or start/center/end. */
    public function scaleType(ScaleType|string $value, int|float|string|null $alignment = null): static
    {
        $this->set('scaleType', $value);
        if ($alignment === null) {
            return $this;
        }

        return $this->set('scaleAlignment', \is_string($alignment)
            ? (self::ALIGNMENTS[$alignment] ?? throw new \InvalidArgumentException("unknown alignment \"$alignment\""))
            : $alignment);
    }

    public function preserveAspectRatio(bool $value = true): static { return $this->set('preserveAspectRatio', $value); }
    public function flipHorizontal(bool $value = true): static { return $this->set('flipHorizontal', $value); }
    public function flipVertical(bool $value = true): static { return $this->set('flipVertical', $value); }

    /** The letterbox colour behind a `contain` image. */
    public function fill(string $color): static { return $this->set('fill', $color); }

    /** An SVG path the image is clipped to. */
    public function clipPath(string $path): static { return $this->set('clipPath', $path); }
}

/** An SVG path. */
final class PathNode extends Node
{
    use LayoutProps;

    public function __construct(string $d)
    {
        parent::__construct('path');
        $this->set('d', $d);
    }

    public function stroke(string $color): static { return $this->set('stroke', $color); }
    public function strokeWidth(int|float $value): static { return $this->set('strokeWidth', $value); }
    public function strokeLineCap(StrokeCap|string $value): static { return $this->set('strokeLineCap', $value); }
    public function strokeLineJoin(StrokeJoin|string $value): static { return $this->set('strokeLineJoin', $value); }
    public function strokeMiterLimit(int|float $value): static { return $this->set('strokeMiterLimit', $value); }
    public function strokeDashArray(int|float ...$values): static { return $this->set('strokeDashArray', $values); }
    public function strokeDashOffset(int|float $value): static { return $this->set('strokeDashOffset', $value); }
    public function fill(string $color): static { return $this->set('fill', $color); }
    public function fillOpacity(int|float $value): static { return $this->set('fillOpacity', $value); }
    public function fillRule(FillRule|string $value): static { return $this->set('fillRule', $value); }

    /** Scale the path data itself, before layout. */
    public function scalePath(int|float $value): static { return $this->set('scalePath', $value); }
}

/** A table. Children are rows. */
final class TableNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$rows)
    {
        parent::__construct('table');
        $this->adopt($rows);
    }

    /** Row and column spacing. One argument sets both. */
    public function spacing(int|float $row, int|float|null $column = null): static
    {
        return $this->set('spacing', [$row, $column ?? $row]);
    }
}

final class TableRowNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$cells)
    {
        parent::__construct('table-row');
        $this->adopt($cells);
    }
}

final class TableCellNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$children)
    {
        parent::__construct('table-cell');
        $this->adopt($children);
    }

    public function colspan(int $value): static { return $this->set('colspan', $value); }
    public function rowspan(int $value): static { return $this->set('rowspan', $value); }
}

/** A bulleted or numbered list. */
final class ListNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$items)
    {
        parent::__construct('list');
        $this->adopt($items);
    }

    /** `disc`, `circle`, `square`, `decimal`, `dash`, `none`, literal text, or a styled marker node. */
    public function listStyle(string|Node $value): static { return $this->set('listStyle', $value); }
    public function markerGap(int|float $value): static { return $this->set('markerGap', $value); }
    public function markerOffset(int|float $value): static { return $this->set('markerOffset', $value); }
    public function startIndex(int $value): static { return $this->set('startIndex', $value); }
}

final class ListItemNode extends Node
{
    use LayoutProps;

    public function __construct(Node|null ...$children)
    {
        parent::__construct('list-item');
        $this->adopt($children);
    }

    /** Override the list's marker for this item alone. */
    public function marker(Node $value): static { return $this->set('marker', $value); }
}

/** Clips every child to an SVG path. */
final class ClipGroupNode extends Node
{
    use LayoutProps;

    public function __construct(string $clipPath, Node|null ...$children)
    {
        parent::__construct('clip-group');
        $this->set('clipPath', $clipPath);
        $this->adopt($children);
    }
}
