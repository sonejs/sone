<?php

declare(strict_types=1);

namespace Sone;

/**
 * The node factories.
 *
 *     use function Sone\{Column, Row, Text, Span};
 *
 * PHP keeps functions and classes in separate symbol tables, so a factory and
 * its class could share a name — but the `Node` suffix is kept for consistency
 * with the other bindings, and because `list` is a reserved word and could not
 * have been a class name anyway.
 */

/** A vertical container. Null children are dropped, so `$flag ? Foo() : null` works. */
function Column(Node|null ...$children): ColumnNode
{
    return new ColumnNode(...$children);
}

/** A horizontal container. */
function Row(Node|null ...$children): RowNode
{
    return new RowNode(...$children);
}

/** A grid container with row-major auto placement. */
function Grid(Node|null ...$children): GridNode
{
    return new GridNode(...$children);
}

/** A paragraph of strings and spans. */
function Text(string|Node ...$content): TextNode
{
    return new TextNode(...$content);
}

/** A styled run inside a `Text`. */
function Span(string $text = ''): SpanNode
{
    return new SpanNode($text);
}

/** Cascade text styling onto every descendant. */
function TextDefault(Node|null ...$children): TextDefaultNode
{
    return new TextDefaultNode(...$children);
}

/** An image, from a path, a URL, or `asset:name`. */
function Photo(string $src): PhotoNode
{
    return new PhotoNode($src);
}

/** An image from raw bytes, inlined into the document as a data URL. */
function PhotoBytes(string $data): PhotoNode
{
    return new PhotoNode('data:application/octet-stream;base64,' . base64_encode($data));
}

/** An SVG path. */
function Path(string $d): PathNode
{
    return new PathNode($d);
}

function Table(Node|null ...$rows): TableNode
{
    return new TableNode(...$rows);
}

function TableRow(Node|null ...$cells): TableRowNode
{
    return new TableRowNode(...$cells);
}

function TableCell(Node|null ...$children): TableCellNode
{
    return new TableCellNode(...$children);
}

/** A bulleted or numbered list. Not `List()`: `list` is a reserved word. */
function BulletList(Node|null ...$items): ListNode
{
    return new ListNode(...$items);
}

function ListItem(Node|null ...$children): ListItemNode
{
    return new ListItemNode(...$children);
}

/** Clip every child to an SVG path. */
function ClipGroup(string $clipPath, Node|null ...$children): ClipGroupNode
{
    return new ClipGroupNode($clipPath, ...$children);
}

/** An explicit page break. Only meaningful with `pageHeight` set. */
function PageBreak(): ColumnNode
{
    return (new ColumnNode())->height(0)->pageBreak(PageBreakMode::Before);
}
