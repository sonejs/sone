namespace Sone;

// Keyword values are structs rather than enums so a string still works: the IR
// accepts a fixed vocabulary, but a binding should never be the reason a value
// the engine understands cannot be expressed. `Justify.SpaceBetween` and
// `"space-between"` compile to the same thing.

/// <summary>How wrapped flex lines are distributed on the cross axis.</summary>
public readonly record struct AlignContent(string Value)
{
    public static readonly AlignContent FlexStart = new("flex-start");
    public static readonly AlignContent FlexEnd = new("flex-end");
    public static readonly AlignContent Center = new("center");
    public static readonly AlignContent Stretch = new("stretch");
    public static readonly AlignContent SpaceBetween = new("space-between");
    public static readonly AlignContent SpaceAround = new("space-around");
    public static readonly AlignContent SpaceEvenly = new("space-evenly");

    public static implicit operator AlignContent(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Cross-axis alignment. Also the type of `AlignSelf`.</summary>
public readonly record struct AlignItems(string Value)
{
    public static readonly AlignItems FlexStart = new("flex-start");
    public static readonly AlignItems FlexEnd = new("flex-end");
    public static readonly AlignItems Center = new("center");
    public static readonly AlignItems Stretch = new("stretch");
    public static readonly AlignItems Baseline = new("baseline");

    public static implicit operator AlignItems(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Main-axis distribution.</summary>
public readonly record struct JustifyContent(string Value)
{
    public static readonly JustifyContent FlexStart = new("flex-start");
    public static readonly JustifyContent FlexEnd = new("flex-end");
    public static readonly JustifyContent Center = new("center");
    public static readonly JustifyContent SpaceBetween = new("space-between");
    public static readonly JustifyContent SpaceAround = new("space-around");
    public static readonly JustifyContent SpaceEvenly = new("space-evenly");

    public static implicit operator JustifyContent(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>The main axis of a container.</summary>
public readonly record struct FlexDirection(string Value)
{
    public static readonly FlexDirection Row = new("row");
    public static readonly FlexDirection Column = new("column");
    public static readonly FlexDirection RowReverse = new("row-reverse");
    public static readonly FlexDirection ColumnReverse = new("column-reverse");

    public static implicit operator FlexDirection(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Whether flex items wrap onto new lines.</summary>
public readonly record struct FlexWrap(string Value)
{
    public static readonly FlexWrap Wrap = new("wrap");
    public static readonly FlexWrap NoWrap = new("nowrap");
    public static readonly FlexWrap WrapReverse = new("wrap-reverse");

    public static implicit operator FlexWrap(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Whether width and height include padding and border.</summary>
public readonly record struct BoxSizing(string Value)
{
    public static readonly BoxSizing BorderBox = new("border-box");
    public static readonly BoxSizing ContentBox = new("content-box");

    public static implicit operator BoxSizing(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Writing direction. Also the type of `TextDir`.</summary>
public readonly record struct Direction(string Value)
{
    public static readonly Direction Ltr = new("ltr");
    public static readonly Direction Rtl = new("rtl");

    public static implicit operator Direction(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>How a node participates in layout.</summary>
public readonly record struct Display(string Value)
{
    public static readonly Display None = new("none");
    public static readonly Display Flex = new("flex");
    public static readonly Display Contents = new("contents");

    public static implicit operator Display(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>What happens to content past a node's box.</summary>
public readonly record struct Overflow(string Value)
{
    public static readonly Overflow Visible = new("visible");
    public static readonly Overflow Hidden = new("hidden");
    public static readonly Overflow Scroll = new("scroll");

    public static implicit operator Overflow(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Positioning scheme.</summary>
public readonly record struct Position(string Value)
{
    public static readonly Position Absolute = new("absolute");
    public static readonly Position Relative = new("relative");
    public static readonly Position Static = new("static");

    public static implicit operator Position(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Where a page break may or must fall. The `PageBreak()` factory emits `Before`.</summary>
public readonly record struct PageBreakMode(string Value)
{
    public static readonly PageBreakMode Before = new("before");
    public static readonly PageBreakMode After = new("after");
    public static readonly PageBreakMode Avoid = new("avoid");

    public static implicit operator PageBreakMode(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>The shape a corner radius produces.</summary>
public readonly record struct Corner(string Value)
{
    public static readonly Corner Cut = new("cut");
    public static readonly Corner Round = new("round");

    public static implicit operator Corner(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>How a photo fills its box.</summary>
public readonly record struct ScaleType(string Value)
{
    public static readonly ScaleType Cover = new("cover");
    public static readonly ScaleType Fill = new("fill");
    public static readonly ScaleType Contain = new("contain");

    public static implicit operator ScaleType(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Roman or slanted.</summary>
public readonly record struct FontStyle(string Value)
{
    public static readonly FontStyle Normal = new("normal");
    public static readonly FontStyle Italic = new("italic");
    public static readonly FontStyle Oblique = new("oblique");

    public static implicit operator FontStyle(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>The line-breaking algorithm.</summary>
public readonly record struct LineBreakMode(string Value)
{
    public static readonly LineBreakMode Greedy = new("greedy");
    public static readonly LineBreakMode KnuthPlass = new("knuth-plass");

    public static implicit operator LineBreakMode(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>What a clipped paragraph ends with.</summary>
public readonly record struct TextOverflow(string Value)
{
    public static readonly TextOverflow Clip = new("clip");
    public static readonly TextOverflow Ellipsis = new("ellipsis");

    public static implicit operator TextOverflow(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Horizontal alignment inside a paragraph.</summary>
public readonly record struct TextAlign(string Value)
{
    public static readonly TextAlign Left = new("left");
    public static readonly TextAlign Right = new("right");
    public static readonly TextAlign Center = new("center");
    public static readonly TextAlign Justify = new("justify");

    public static implicit operator TextAlign(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Greedy wrapping, or ragged-edge balancing.</summary>
public readonly record struct TextWrap(string Value)
{
    public static readonly TextWrap Wrap = new("wrap");
    public static readonly TextWrap Balance = new("balance");

    public static implicit operator TextWrap(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>How an open path ends.</summary>
public readonly record struct StrokeCap(string Value)
{
    public static readonly StrokeCap Butt = new("butt");
    public static readonly StrokeCap Round = new("round");
    public static readonly StrokeCap Square = new("square");

    public static implicit operator StrokeCap(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>How two path segments meet.</summary>
public readonly record struct StrokeJoin(string Value)
{
    public static readonly StrokeJoin Bevel = new("bevel");
    public static readonly StrokeJoin Miter = new("miter");
    public static readonly StrokeJoin Round = new("round");

    public static implicit operator StrokeJoin(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Which regions of a self-intersecting path are inside it.</summary>
public readonly record struct FillRule(string Value)
{
    public static readonly FillRule EvenOdd = new("evenodd");
    public static readonly FillRule NonZero = new("nonzero");

    public static implicit operator FillRule(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>The paragraph's base direction for bidi resolution.</summary>
public readonly record struct BaseDir(string Value)
{
    public static readonly BaseDir Ltr = new("ltr");
    public static readonly BaseDir Rtl = new("rtl");
    public static readonly BaseDir Auto = new("auto");

    public static implicit operator BaseDir(string value) => new(value);

    public override string ToString() => Value;
}

/// <summary>Whether the final page is a full page or shrinks to its content.</summary>
public readonly record struct LastPageHeight(string Value)
{
    public static readonly LastPageHeight Uniform = new("uniform");
    public static readonly LastPageHeight Content = new("content");

    public static implicit operator LastPageHeight(string value) => new(value);

    public override string ToString() => Value;
}
