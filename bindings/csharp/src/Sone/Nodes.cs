namespace Sone;

/// <summary>A vertical container.</summary>
public sealed class ColumnNode : Node, ILayoutNode
{
    internal ColumnNode() : base("column") { }
}

/// <summary>A horizontal container.</summary>
public sealed class RowNode : Node, ILayoutNode
{
    internal RowNode() : base("row") { }
}

/// <summary>A grid container with row-major auto placement.</summary>
public sealed class GridNode : Node, ILayoutNode
{
    internal GridNode() : base("grid") { }

    /// <summary>Explicit column tracks.</summary>
    public GridNode Columns(params Track[] tracks) => SetTracks("columns", tracks);

    /// <summary>Explicit row tracks.</summary>
    public GridNode Rows(params Track[] tracks) => SetTracks("rows", tracks);

    /// <summary>The track size for rows created by auto placement.</summary>
    public GridNode AutoRows(params Track[] tracks) => SetTracks("autoRows", tracks);

    /// <summary>The track size for columns created by auto placement.</summary>
    public GridNode AutoColumns(params Track[] tracks) => SetTracks("autoColumns", tracks);

    private GridNode SetTracks(string key, Track[] tracks)
    {
        var values = new List<object?>(tracks.Length);
        foreach (var track in tracks)
        {
            values.Add(track.ToIr());
        }
        Props.Set(key, values);
        return this;
    }
}

/// <summary>A styled run inside a <see cref="TextNode"/>.</summary>
public sealed class SpanNode : Node, ISpanStyle
{
    internal SpanNode() : base("span") { }
}

/// <summary>
/// A paragraph. Both a box and a run of text, so it carries layout, span and
/// paragraph properties at once.
/// </summary>
public sealed class TextNode : Node, ILayoutNode, ISpanStyle, ITextBlock
{
    internal TextNode() : base("text") { }

    /// <summary>
    /// The font size — not the box size. This mirrors the TypeScript API, where
    /// <c>TextPropsBuilder</c> omits the layout <c>size</c>, and being an
    /// instance method it also wins over the two extension methods that would
    /// otherwise both apply here.
    /// </summary>
    public TextNode Size(double value)
    {
        Props.Set("size", value);
        return this;
    }

    /// <summary>Whether the paragraph wraps. Not the flexbox <c>flexWrap</c>.</summary>
    public TextNode Wrap(bool wrap = true)
    {
        Props.Set("nowrap", !wrap);
        return this;
    }
}

/// <summary>Cascades text styling onto its descendants without drawing a box.</summary>
public sealed class TextDefaultNode : Node, ISpanStyle, ITextBlock
{
    internal TextDefaultNode() : base("text-default") { }
}

/// <summary>An image.</summary>
public sealed class PhotoNode : Node, ILayoutNode
{
    internal PhotoNode() : base("photo") { }

    /// <summary>
    /// How the image fills its box. <paramref name="alignment"/> is 0..1, or one
    /// of <c>"start"</c>, <c>"center"</c>, <c>"end"</c>.
    /// </summary>
    public PhotoNode ScaleType(ScaleType value, double? alignment = null)
    {
        Props.Set("scaleType", value.Value);
        Props.Set("scaleAlignment", alignment);
        return this;
    }

    /// <inheritdoc cref="ScaleType(Sone.ScaleType,double?)"/>
    public PhotoNode ScaleType(ScaleType value, string alignment) => ScaleType(value, alignment switch
    {
        "start" => 0.0,
        "center" => 0.5,
        "end" => 1.0,
        _ => throw new ArgumentException($"unknown alignment \"{alignment}\"", nameof(alignment)),
    });

    public PhotoNode PreserveAspectRatio(bool value = true)
    {
        Props.Set("preserveAspectRatio", value);
        return this;
    }

    public PhotoNode FlipHorizontal(bool value = true)
    {
        Props.Set("flipHorizontal", value);
        return this;
    }

    public PhotoNode FlipVertical(bool value = true)
    {
        Props.Set("flipVertical", value);
        return this;
    }

    /// <summary>The letterbox colour behind a <c>contain</c> image.</summary>
    public PhotoNode Fill(string color)
    {
        Props.Set("fill", color);
        return this;
    }

    /// <summary>An SVG path the image is clipped to.</summary>
    public PhotoNode ClipPath(string path)
    {
        Props.Set("clipPath", path);
        return this;
    }
}

/// <summary>An SVG path. Named <c>SvgPath</c> because <c>Path</c> is <c>System.IO.Path</c>.</summary>
public sealed class SvgPathNode : Node, ILayoutNode
{
    internal SvgPathNode() : base("path") { }

    public SvgPathNode Stroke(string color)
    {
        Props.Set("stroke", color);
        return this;
    }

    public SvgPathNode StrokeWidth(double value)
    {
        Props.Set("strokeWidth", value);
        return this;
    }

    public SvgPathNode StrokeLineCap(StrokeCap value)
    {
        Props.Set("strokeLineCap", value.Value);
        return this;
    }

    public SvgPathNode StrokeLineJoin(StrokeJoin value)
    {
        Props.Set("strokeLineJoin", value.Value);
        return this;
    }

    public SvgPathNode StrokeMiterLimit(double value)
    {
        Props.Set("strokeMiterLimit", value);
        return this;
    }

    public SvgPathNode StrokeDashArray(params double[] values)
    {
        Props.Set("strokeDashArray", Box(values));
        return this;
    }

    public SvgPathNode StrokeDashOffset(double value)
    {
        Props.Set("strokeDashOffset", value);
        return this;
    }

    public SvgPathNode Fill(string color)
    {
        Props.Set("fill", color);
        return this;
    }

    public SvgPathNode FillOpacity(double value)
    {
        Props.Set("fillOpacity", value);
        return this;
    }

    public SvgPathNode FillRule(FillRule value)
    {
        Props.Set("fillRule", value.Value);
        return this;
    }

    /// <summary>Scale the path data itself, before layout.</summary>
    public SvgPathNode ScalePath(double value)
    {
        Props.Set("scalePath", value);
        return this;
    }

    private static List<object?> Box(double[] values)
    {
        var list = new List<object?>(values.Length);
        foreach (var value in values)
        {
            list.Add(value);
        }
        return list;
    }
}

/// <summary>A table. Children are <see cref="TableRowNode"/>s.</summary>
public sealed class TableNode : Node, ILayoutNode
{
    internal TableNode() : base("table") { }

    /// <summary>Row and column spacing: one value for both, or two.</summary>
    public TableNode Spacing(double row, double? column = null)
    {
        Props.Set("spacing", new List<object?> { row, column ?? row });
        return this;
    }
}

/// <summary>A table row. Children are <see cref="TableCellNode"/>s.</summary>
public sealed class TableRowNode : Node, ILayoutNode
{
    internal TableRowNode() : base("table-row") { }
}

/// <summary>A table cell.</summary>
public sealed class TableCellNode : Node, ILayoutNode
{
    internal TableCellNode() : base("table-cell") { }

    public TableCellNode Colspan(int value)
    {
        Props.Set("colspan", value);
        return this;
    }

    public TableCellNode Rowspan(int value)
    {
        Props.Set("rowspan", value);
        return this;
    }
}

/// <summary>A bulleted or numbered list.</summary>
public sealed class ListNode : Node, ILayoutNode
{
    internal ListNode() : base("list") { }

    /// <summary><c>disc</c>, <c>circle</c>, <c>square</c>, <c>decimal</c>, <c>dash</c>, <c>none</c>, or literal marker text.</summary>
    public ListNode ListStyle(string value)
    {
        Props.Set("listStyle", value);
        return this;
    }

    /// <summary>A styled marker node. <c>{}</c> in its text is replaced with the item number.</summary>
    public ListNode ListStyle(INode marker)
    {
        Props.Set("listStyle", marker);
        return this;
    }

    /// <summary>The gap between a marker and its item.</summary>
    public ListNode MarkerGap(double value)
    {
        Props.Set("markerGap", value);
        return this;
    }

    /// <summary>Vertical nudge for the marker, to sit it on the first baseline.</summary>
    public ListNode MarkerOffset(double value)
    {
        Props.Set("markerOffset", value);
        return this;
    }

    /// <summary>The number the first item counts from.</summary>
    public ListNode StartIndex(int value)
    {
        Props.Set("startIndex", value);
        return this;
    }
}

/// <summary>One item in a <see cref="ListNode"/>.</summary>
public sealed class ListItemNode : Node, ILayoutNode
{
    internal ListItemNode() : base("list-item") { }

    /// <summary>Override the list's marker for this item alone.</summary>
    public ListItemNode Marker(INode marker)
    {
        Props.Set("marker", marker);
        return this;
    }
}

/// <summary>Clips every child to an SVG path.</summary>
public sealed class ClipGroupNode : Node, ILayoutNode
{
    internal ClipGroupNode() : base("clip-group") { }

    public ClipGroupNode ClipPath(string path)
    {
        Props.Set("clipPath", path);
        return this;
    }
}
