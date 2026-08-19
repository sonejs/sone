namespace Sone;

/// <summary>
/// The node factories. Add <c>using static Sone.Dsl;</c> and the tree reads the
/// way it does in TypeScript:
/// <code>
/// var root = Column(
///     Text("Hello").Size(28).Weight(FontWeight.Bold),
///     Row(
///         Column().Bg("salmon").Size(50).Rounded(14),
///         Column().Bg("orange").Size(50).Rounded(14)
///     ).Gap(10)
/// ).Gap(20).Padding(20).Bg("khaki").CornerRadius(28);
/// </code>
/// </summary>
/// <remarks>
/// The factory and the class it returns are named separately —
/// <c>Column()</c> returns a <see cref="ColumnNode"/> — because a static method
/// and a type cannot share a name and still be invocable. It is the same split
/// the Python binding uses, and it happens to defuse most BCL collisions too;
/// the one that survives is <c>Path</c>, which is <see cref="System.IO.Path"/>,
/// so the factory here is <see cref="SvgPath"/>.
/// </remarks>
public static class Dsl
{
    /// <summary>A vertical container. Null children are dropped, so <c>cond ? Foo() : null</c> works.</summary>
    public static ColumnNode Column(params INode?[] children) => Build(new ColumnNode(), children);

    /// <summary>A horizontal container.</summary>
    public static RowNode Row(params INode?[] children) => Build(new RowNode(), children);

    /// <summary>A grid container with row-major auto placement.</summary>
    public static GridNode Grid(params INode?[] children) => Build(new GridNode(), children);

    /// <summary>A paragraph of strings and <see cref="SpanNode"/>s.</summary>
    public static TextNode Text(params Inline[] content)
    {
        var node = new TextNode();
        foreach (var inline in content)
        {
            node.InlineContent.Add(inline);
        }
        return node;
    }

    /// <summary>A styled run inside a <see cref="TextNode"/>.</summary>
    public static SpanNode Span(string text)
    {
        var node = new SpanNode();
        node.InlineContent.Add(text);
        return node;
    }

    /// <summary>Cascade text styling onto every descendant, drawing no box.</summary>
    public static TextDefaultNode TextDefault(params INode?[] children) =>
        Build(new TextDefaultNode(), children);

    /// <summary>An image, from a path, a URL, or <c>asset:name</c>.</summary>
    public static PhotoNode Photo(string src)
    {
        var node = new PhotoNode();
        node.Props.Set("src", src);
        return node;
    }

    /// <summary>An image from raw bytes, inlined into the document as a data URL.</summary>
    public static PhotoNode Photo(ReadOnlySpan<byte> data) =>
        Photo("data:application/octet-stream;base64," + Convert.ToBase64String(data));

    /// <summary>An SVG path. Named for <see cref="System.IO.Path"/>, which owns <c>Path</c>.</summary>
    public static SvgPathNode SvgPath(string d)
    {
        var node = new SvgPathNode();
        node.Props.Set("d", d);
        return node;
    }

    /// <summary>A table. Children are rows.</summary>
    public static TableNode Table(params INode?[] rows) => Build(new TableNode(), rows);

    /// <summary>A table row. Children are cells.</summary>
    public static TableRowNode TableRow(params INode?[] cells) => Build(new TableRowNode(), cells);

    /// <summary>A table cell.</summary>
    public static TableCellNode TableCell(params INode?[] children) => Build(new TableCellNode(), children);

    /// <summary>A bulleted or numbered list. Children are items.</summary>
    public static ListNode List(params INode?[] items) => Build(new ListNode(), items);

    /// <summary>One item in a list.</summary>
    public static ListItemNode ListItem(params INode?[] children) => Build(new ListItemNode(), children);

    /// <summary>Clip every child to an SVG path.</summary>
    public static ClipGroupNode ClipGroup(string path, params INode?[] children)
    {
        var node = Build(new ClipGroupNode(), children);
        node.Props.Set("clipPath", path);
        return node;
    }

    /// <summary>An explicit page break. Only meaningful with <c>pageHeight</c> set.</summary>
    public static ColumnNode PageBreak() =>
        Column().Height(0).PageBreak(PageBreakMode.Before);

    private static T Build<T>(T node, INode?[] children) where T : Node
    {
        node.Adopt(children);
        return node;
    }
}
