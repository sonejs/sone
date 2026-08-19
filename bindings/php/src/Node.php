<?php

declare(strict_types=1);

namespace Sone;

/** A length: a number, `'auto'`, or a percentage such as `'50%'`. */
// phpcs:ignore
final class Dim
{
    public static function check(int|float|string $value, string $property): int|float|string
    {
        if (\is_string($value) && $value !== 'auto' && !preg_match('/^-?[\d.]+%?$/', $value)) {
            throw new \InvalidArgumentException("$property: expected a number, \"auto\", or a percentage, got \"$value\"");
        }

        return $value;
    }
}

/**
 * A node in the document tree.
 *
 * Every setter returns `static`, so a chain keeps the concrete type and PHP's
 * named arguments work throughout: `->padding(top: 20, left: 4)`.
 */
abstract class Node implements \JsonSerializable
{
    /** @var array<string,mixed> */
    public array $props = [];

    /** @var list<Node> */
    public array $children = [];

    /** @var list<string|Node> */
    public array $inline = [];

    public function __construct(public readonly string $type) {}

    /** A name for this node, echoed back by `layout()` and `metadata()`. */
    public function tag(string $value): static
    {
        return $this->set('tag', $value);
    }

    /**
     * Set raw IR properties, for anything this API does not cover yet.
     *
     * @param array<string,mixed> $values
     */
    public function apply(array $values): static
    {
        foreach ($values as $key => $value) {
            $this->props[$key] = $value;
        }

        return $this;
    }

    /** @internal */
    public function set(string $key, mixed $value): static
    {
        if ($value !== null) {
            $this->props[$key] = $value instanceof \BackedEnum ? $value->value : $value;
        }

        return $this;
    }

    /**
     * Set a property that may legitimately be null — an explicit null clears a
     * decoration colour, which the engine reads differently from unset.
     *
     * @internal
     */
    public function setNullable(string $key, mixed $value): static
    {
        $this->props[$key] = $value instanceof \BackedEnum ? $value->value : $value;

        return $this;
    }

    /** @internal @param list<mixed> $values */
    public function push(string $key, array $values): static
    {
        $this->props[$key] = [...($this->props[$key] ?? []), ...array_map(
            static fn (mixed $value): mixed => $value instanceof \BackedEnum ? $value->value : $value,
            $values,
        )];

        return $this;
    }

    /** This node as an IR document fragment. */
    public function toIr(): array
    {
        $out = ['type' => $this->type];
        if ($this->props !== []) {
            $out['props'] = array_map(
                static fn (mixed $value): mixed => $value instanceof Node ? $value->toIr() : $value,
                $this->props,
            );
        }
        if ($this->children !== []) {
            $out['children'] = array_map(static fn (Node $child): array => $child->toIr(), $this->children);
        }
        if ($this->inline !== []) {
            $out['inline'] = array_map(
                static fn (string|Node $item): string|array => $item instanceof Node ? $item->toIr() : $item,
                $this->inline,
            );
        }

        return $out;
    }

    public function jsonSerialize(): array
    {
        return $this->toIr();
    }

    /** @param iterable<Node|null> $children */
    protected function adopt(iterable $children): void
    {
        foreach ($children as $child) {
            if ($child !== null) {
                $this->children[] = $child;
            }
        }
    }
}

/** Flexbox, sizing, spacing and the visual box properties. */
trait LayoutProps
{
    public function alignContent(AlignContent|string $value): static { return $this->set('alignContent', $value); }
    public function alignItems(AlignItems|string $value): static { return $this->set('alignItems', $value); }
    public function alignSelf(AlignItems|string $value): static { return $this->set('alignSelf', $value); }
    public function aspectRatio(int|float $value): static { return $this->set('aspectRatio', $value); }
    public function boxSizing(BoxSizing|string $value): static { return $this->set('boxSizing', $value); }
    public function direction(Direction|string $value): static { return $this->set('direction', $value); }
    public function display(Display|string $value): static { return $this->set('display', $value); }
    public function flex(int|float $value): static { return $this->set('flex', $value); }
    public function basis(int|float|string $value): static { return $this->set('flexBasis', Dim::check($value, 'basis')); }
    public function flexDirection(FlexDirection|string $value): static { return $this->set('flexDirection', $value); }
    public function grow(int|float $value): static { return $this->set('flexGrow', $value); }
    public function shrink(int|float $value): static { return $this->set('flexShrink', $value); }
    public function wrap(FlexWrap|string $value): static { return $this->set('flexWrap', $value); }
    public function justifyContent(JustifyContent|string $value): static { return $this->set('justifyContent', $value); }
    public function overflow(Overflow|string $value): static { return $this->set('overflow', $value); }
    public function position(Position|string $value): static { return $this->set('position', $value); }

    public function gap(int|float $value): static { return $this->set('gap', $value); }
    public function rowGap(int|float $value): static { return $this->set('rowGap', $value); }
    public function columnGap(int|float $value): static { return $this->set('columnGap', $value); }

    /** Width and height. One argument makes a square. */
    public function size(int|float|string $width, int|float|string|null $height = null): static
    {
        return $this->set('width', Dim::check($width, 'size'))
            ->set('height', Dim::check($height ?? $width, 'size'));
    }

    public function width(int|float|string $value): static { return $this->set('width', Dim::check($value, 'width')); }
    public function height(int|float|string $value): static { return $this->set('height', Dim::check($value, 'height')); }
    public function minWidth(int|float|string $value): static { return $this->set('minWidth', Dim::check($value, 'minWidth')); }
    public function minHeight(int|float|string $value): static { return $this->set('minHeight', Dim::check($value, 'minHeight')); }
    public function maxWidth(int|float|string $value): static { return $this->set('maxWidth', Dim::check($value, 'maxWidth')); }
    public function maxHeight(int|float|string $value): static { return $this->set('maxHeight', Dim::check($value, 'maxHeight')); }

    /**
     * CSS 1-4 value shorthand. An omitted side follows CSS — right defaults to
     * top, bottom to top, left to right — so `padding(top: 20)` behaves.
     */
    public function padding(
        int|float|string $top,
        int|float|string|null $right = null,
        int|float|string|null $bottom = null,
        int|float|string|null $left = null,
    ): static {
        return $this->box(['padding', 'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft'], $top, $right, $bottom, $left);
    }

    public function margin(
        int|float|string $top,
        int|float|string|null $right = null,
        int|float|string|null $bottom = null,
        int|float|string|null $left = null,
    ): static {
        return $this->box(['margin', 'marginTop', 'marginRight', 'marginBottom', 'marginLeft'], $top, $right, $bottom, $left);
    }

    public function borderWidth(
        int|float $top,
        int|float|null $right = null,
        int|float|null $bottom = null,
        int|float|null $left = null,
    ): static {
        return $this->box(['borderWidth', 'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth'], $top, $right, $bottom, $left);
    }

    public function borderColor(string $value): static { return $this->set('borderColor', $value); }
    public function marginTop(int|float|string $value): static { return $this->set('marginTop', Dim::check($value, 'marginTop')); }
    public function marginRight(int|float|string $value): static { return $this->set('marginRight', Dim::check($value, 'marginRight')); }
    public function marginBottom(int|float|string $value): static { return $this->set('marginBottom', Dim::check($value, 'marginBottom')); }
    public function marginLeft(int|float|string $value): static { return $this->set('marginLeft', Dim::check($value, 'marginLeft')); }
    public function paddingTop(int|float|string $value): static { return $this->set('paddingTop', Dim::check($value, 'paddingTop')); }
    public function paddingRight(int|float|string $value): static { return $this->set('paddingRight', Dim::check($value, 'paddingRight')); }
    public function paddingBottom(int|float|string $value): static { return $this->set('paddingBottom', Dim::check($value, 'paddingBottom')); }
    public function paddingLeft(int|float|string $value): static { return $this->set('paddingLeft', Dim::check($value, 'paddingLeft')); }

    public function top(int|float|string $value): static { return $this->set('top', Dim::check($value, 'top')); }
    public function right(int|float|string $value): static { return $this->set('right', Dim::check($value, 'right')); }
    public function bottom(int|float|string $value): static { return $this->set('bottom', Dim::check($value, 'bottom')); }
    public function left(int|float|string $value): static { return $this->set('left', Dim::check($value, 'left')); }

    /** The leading inset, which flips with the writing direction. */
    public function start(int|float|string $value): static { return $this->set('start', Dim::check($value, 'start')); }

    /** The trailing inset, which flips with the writing direction. */
    public function end(int|float|string $value): static { return $this->set('end', Dim::check($value, 'end')); }
    public function inset(int|float|string $value): static { return $this->set('inset', Dim::check($value, 'inset')); }

    public function gridColumn(int $start, ?int $span = null): static
    {
        return $this->set('gridColumnStart', $start)->set('gridColumnSpan', $span);
    }

    public function gridRow(int $start, ?int $span = null): static
    {
        return $this->set('gridRowStart', $start)->set('gridRowSpan', $span);
    }

    /** Force or forbid a page break at this node. Needs `pageHeight`. */
    public function pageBreak(PageBreakMode|string $value): static { return $this->set('pageBreak', $value); }

    public function translateX(int|float $value): static { return $this->set('translateX', $value); }
    public function translateY(int|float $value): static { return $this->set('translateY', $value); }

    /** Rotation in degrees, about the node's centre. */
    public function rotate(int|float $degrees): static { return $this->set('rotation', $degrees); }

    /** Scale. One argument scales both axes. */
    public function scale(int|float $x, int|float|null $y = null): static
    {
        return $this->set('scale', [$x, $y ?? $x]);
    }

    /** Add background layers: CSS colours, gradients, or a `Photo`. */
    public function bg(string|Node ...$layers): static { return $this->push('background', $layers); }
    public function background(string|Node ...$layers): static { return $this->push('background', $layers); }

    public function opacity(int|float $value): static { return $this->set('opacity', $value); }

    /** Corner radii: one value for all four, or up to four clockwise from the top left. */
    public function cornerRadius(int|float ...$radii): static { return $this->set('cornerRadius', $radii); }
    public function rounded(int|float ...$radii): static { return $this->set('cornerRadius', $radii); }
    public function borderRadius(int|float ...$radii): static { return $this->set('cornerRadius', $radii); }

    /** Squircle-ness, 0..1. Figma's corner smoothing. */
    public function cornerSmoothing(int|float $value): static { return $this->set('cornerSmoothing', $value); }
    public function borderSmoothing(int|float $value): static { return $this->set('cornerSmoothing', $value); }
    public function corner(Corner|string $value): static { return $this->set('corner', $value); }

    /** Add CSS `box-shadow` strings. */
    public function shadow(string ...$shadows): static { return $this->push('shadows', $shadows); }

    // CSS filters, applied in the order they are added.
    public function blur(int|float $radius): static { return $this->filter("blur({$this->num($radius)}px)"); }
    public function brightness(int|float $amount): static { return $this->filter("brightness({$this->num($amount)})"); }
    public function contrast(int|float $amount): static { return $this->filter("contrast({$this->num($amount)})"); }
    public function grayscale(int|float $amount): static { return $this->filter("grayscale({$this->num($amount)})"); }
    public function hueRotate(int|float $degrees): static { return $this->filter("hue-rotate({$this->num($degrees)})"); }
    public function invert(int|float $amount): static { return $this->filter("invert({$this->num($amount)})"); }
    public function saturate(int|float $amount): static { return $this->filter("saturate({$this->num($amount)})"); }
    public function sepia(int|float $amount): static { return $this->filter("sepia({$this->num($amount)})"); }

    public function filter(string $css): static { return $this->push('filters', [$css]); }

    private function num(int|float $value): string
    {
        return \is_float($value) && $value === floor($value) ? (string) (int) $value : (string) $value;
    }

    private function box(array $keys, mixed $top, mixed $right, mixed $bottom, mixed $left): static
    {
        if ($right === null && $bottom === null && $left === null) {
            return $this->set($keys[0], $top);
        }

        return $this->set($keys[1], $top)
            ->set($keys[2], $right ?? $top)
            ->set($keys[3], $bottom ?? $top)
            ->set($keys[4], $left ?? $right ?? $top);
    }
}

/** Span-level text styling. */
trait SpanStyleProps
{
    public function color(string $value): static { return $this->set('color', $value); }

    /** The font size, not the box size. */
    public function size(int|float $value): static { return $this->set('size', $value); }

    /** The font stack, in fallback order. */
    public function font(string ...$families): static { return $this->set('font', $families); }

    public function style(FontStyle|string $value): static { return $this->set('style', $value); }

    /** A CSS keyword such as `'bold'`, or a number. */
    public function weight(string|int|float $value): static { return $this->set('weight', $value); }
    public function letterSpacing(int|float $value): static { return $this->set('letterSpacing', $value); }
    public function wordSpacing(int|float $value): static { return $this->set('wordSpacing', $value); }

    public function underline(int|float $thickness = 1.0): static { return $this->set('underline', $thickness); }
    public function overline(int|float $thickness = 1.0): static { return $this->set('overline', $thickness); }
    public function lineThrough(int|float $thickness = 1.0): static { return $this->set('lineThrough', $thickness); }

    /** Pass nothing for an explicit null, which means "use the text colour". */
    public function underlineColor(?string $value = null): static { return $this->setNullable('underlineColor', $value); }
    public function overlineColor(?string $value = null): static { return $this->setNullable('overlineColor', $value); }
    public function lineThroughColor(?string $value = null): static { return $this->setNullable('lineThroughColor', $value); }
    public function highlight(?string $value = null): static { return $this->setNullable('highlightColor', $value); }

    /** Add CSS `text-shadow` strings. */
    public function dropShadow(string ...$shadows): static { return $this->push('dropShadows', $shadows); }

    /** The glyph outline colour. */
    public function strokeColor(string $value): static { return $this->set('strokeColor', $value); }

    /** The glyph outline width. */
    public function strokeWidth(int|float $value): static { return $this->set('strokeWidth', $value); }

    /** Shift the run off its baseline — superscripts, subscripts. */
    public function offsetY(int|float $value): static { return $this->set('offsetY', $value); }

    /** Force this run's direction, overriding bidi resolution. */
    public function textDir(Direction|string $value): static { return $this->set('textDir', $value); }
}

/** Paragraph-level properties. */
trait TextBlockProps
{
    public function nowrap(bool $value = true): static { return $this->set('nowrap', $value); }
    public function maxLines(int|float $value): static { return $this->set('maxLines', $value); }
    public function lineBreak(LineBreakMode|string $value): static { return $this->set('lineBreak', $value); }
    public function textOverflow(TextOverflow|string $value): static { return $this->set('textOverflow', $value); }
    public function lineHeight(int|float $value): static { return $this->set('lineHeight', $value); }
    public function align(TextAlign|string $value): static { return $this->set('align', $value); }
    public function indent(int|float $value): static { return $this->set('indentSize', $value); }
    public function hangingIndent(int|float $value): static { return $this->set('hangingIndentSize', $value); }
    public function tabStops(int|float ...$stops): static { return $this->set('tabStops', $stops); }
    public function tabLeader(string $value): static { return $this->set('tabLeader', $value); }
    public function autofit(bool $value = true): static { return $this->set('autofit', $value); }

    /** Rotation of the text inside its box, in degrees. */
    public function orientation(int $degrees): static { return $this->set('orientation', $degrees); }

    /** Paint the glyphs with an image instead of a colour. */
    public function clipImage(Node $photo): static { return $this->set('clipImage', $photo); }

    /** The base direction used to resolve bidi runs. */
    public function baseDir(BaseDir|string $value): static { return $this->set('baseDir', $value); }

    /** Greedy wrapping, or balancing for a ragged edge. */
    public function textWrap(TextWrap|string $value): static { return $this->set('textWrap', $value); }
}
