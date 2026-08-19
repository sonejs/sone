<?php

declare(strict_types=1);

namespace Sone;

// Backed enums, and every setter that takes one also takes the plain string —
// so completion is there when you want it and a value the engine understands is
// never unreachable.

enum AlignContent: string
{
    case FlexStart = 'flex-start';
    case FlexEnd = 'flex-end';
    case Center = 'center';
    case Stretch = 'stretch';
    case SpaceBetween = 'space-between';
    case SpaceAround = 'space-around';
    case SpaceEvenly = 'space-evenly';
}

enum AlignItems: string
{
    case FlexStart = 'flex-start';
    case FlexEnd = 'flex-end';
    case Center = 'center';
    case Stretch = 'stretch';
    case Baseline = 'baseline';
}

enum JustifyContent: string
{
    case FlexStart = 'flex-start';
    case FlexEnd = 'flex-end';
    case Center = 'center';
    case SpaceBetween = 'space-between';
    case SpaceAround = 'space-around';
    case SpaceEvenly = 'space-evenly';
}

enum FlexDirection: string
{
    case Row = 'row';
    case Column = 'column';
    case RowReverse = 'row-reverse';
    case ColumnReverse = 'column-reverse';
}

enum FlexWrap: string
{
    case Wrap = 'wrap';
    case NoWrap = 'nowrap';
    case WrapReverse = 'wrap-reverse';
}

enum BoxSizing: string
{
    case BorderBox = 'border-box';
    case ContentBox = 'content-box';
}

enum Direction: string
{
    case Ltr = 'ltr';
    case Rtl = 'rtl';
}

enum Display: string
{
    case None = 'none';
    case Flex = 'flex';
    case Contents = 'contents';
}

enum Overflow: string
{
    case Visible = 'visible';
    case Hidden = 'hidden';
    case Scroll = 'scroll';
}

enum Position: string
{
    case Absolute = 'absolute';
    case Relative = 'relative';
    case Static = 'static';
}

enum PageBreakMode: string
{
    case Before = 'before';
    case After = 'after';
    case Avoid = 'avoid';
}

enum Corner: string
{
    case Cut = 'cut';
    case Round = 'round';
}

enum ScaleType: string
{
    case Cover = 'cover';
    case Fill = 'fill';
    case Contain = 'contain';
}

enum FontStyle: string
{
    case Normal = 'normal';
    case Italic = 'italic';
    case Oblique = 'oblique';
}

enum LineBreakMode: string
{
    case Greedy = 'greedy';
    case KnuthPlass = 'knuth-plass';
}

enum TextOverflow: string
{
    case Clip = 'clip';
    case Ellipsis = 'ellipsis';
}

enum TextAlign: string
{
    case Left = 'left';
    case Right = 'right';
    case Center = 'center';
    case Justify = 'justify';
}

enum TextWrap: string
{
    case Wrap = 'wrap';
    case Balance = 'balance';
}

enum StrokeCap: string
{
    case Butt = 'butt';
    case Round = 'round';
    case Square = 'square';
}

enum StrokeJoin: string
{
    case Bevel = 'bevel';
    case Miter = 'miter';
    case Round = 'round';
}

enum FillRule: string
{
    case EvenOdd = 'evenodd';
    case NonZero = 'nonzero';
}

enum BaseDir: string
{
    case Ltr = 'ltr';
    case Rtl = 'rtl';
    case Auto = 'auto';
}

enum LastPageHeight: string
{
    case Uniform = 'uniform';
    case Content = 'content';
}

enum Granularity: string
{
    case Node = 'node';
    case Line = 'line';
    case Word = 'word';
}
/** The output formats the engine can encode. */
enum OutputFormat: string
{
    case Png = 'png';
    case Jpeg = 'jpeg';
    case Webp = 'webp';

    /** Raw RGBA pixels, row-major, unpremultiplied. */
    case Raw = 'raw';

    /** A PDF. With `pageHeight` set, one page per break and selectable text. */
    case Pdf = 'pdf';
    case Svg = 'svg';
}
