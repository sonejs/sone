namespace Sone;

/// <summary>
/// Flexbox, sizing, spacing and the visual box properties, for every node that
/// draws a box.
/// </summary>
/// <remarks>
/// These are generic extension methods rather than instance methods so each one
/// is written once and still returns the caller's own type: <c>T</c> infers to
/// <c>ColumnNode</c>, so <c>Column().Bg("salmon").Size(50)</c> chains exactly.
/// It also lets <see cref="TextNode"/> be a box, a span and a paragraph at once,
/// which single inheritance would not allow.
/// </remarks>
public static class LayoutProps
{
    private static readonly string[] BorderKeys =
        ["borderWidth", "borderTopWidth", "borderRightWidth", "borderBottomWidth", "borderLeftWidth"];

    private static readonly string[] MarginKeys =
        ["margin", "marginTop", "marginRight", "marginBottom", "marginLeft"];

    private static readonly string[] PaddingKeys =
        ["padding", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft"];

    // ── identity ─────────────────────────────────────────────────────────────

    /// <summary>A name for this node, echoed back by <c>Layout()</c> and <c>Metadata()</c>.</summary>
    public static T Tag<T>(this T node, string value) where T : INode
    {
        node.Props.Set("tag", value);
        return node;
    }

    /// <summary>Set raw IR properties, for anything this API does not cover yet.</summary>
    public static T Apply<T>(this T node, IReadOnlyDictionary<string, object?> values) where T : INode
    {
        foreach (var (key, value) in values)
        {
            node.Props.SetNullable(key, value);
        }
        return node;
    }

    // ── flexbox ──────────────────────────────────────────────────────────────

    public static T AlignContent<T>(this T node, AlignContent value) where T : ILayoutNode =>
        node.Set("alignContent", value.Value);

    public static T AlignItems<T>(this T node, AlignItems value) where T : ILayoutNode =>
        node.Set("alignItems", value.Value);

    public static T AlignSelf<T>(this T node, AlignItems value) where T : ILayoutNode =>
        node.Set("alignSelf", value.Value);

    public static T AspectRatio<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("aspectRatio", value);

    public static T BoxSizing<T>(this T node, BoxSizing value) where T : ILayoutNode =>
        node.Set("boxSizing", value.Value);

    public static T Direction<T>(this T node, Direction value) where T : ILayoutNode =>
        node.Set("direction", value.Value);

    public static T Display<T>(this T node, Display value) where T : ILayoutNode =>
        node.Set("display", value.Value);

    public static T Flex<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("flex", value);

    /// <summary>The flex base size, before grow and shrink.</summary>
    public static T Basis<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("flexBasis", value.ToIr());

    public static T FlexDirection<T>(this T node, FlexDirection value) where T : ILayoutNode =>
        node.Set("flexDirection", value.Value);

    public static T Grow<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("flexGrow", value);

    public static T Shrink<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("flexShrink", value);

    /// <summary>Flexbox wrapping. On a <see cref="TextNode"/> use <c>Wrap(bool)</c> for the paragraph.</summary>
    public static T Wrap<T>(this T node, FlexWrap value) where T : ILayoutNode =>
        node.Set("flexWrap", value.Value);

    public static T JustifyContent<T>(this T node, JustifyContent value) where T : ILayoutNode =>
        node.Set("justifyContent", value.Value);

    public static T Overflow<T>(this T node, Overflow value) where T : ILayoutNode =>
        node.Set("overflow", value.Value);

    public static T Position<T>(this T node, Position value) where T : ILayoutNode =>
        node.Set("position", value.Value);

    public static T Gap<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("gap", value);

    public static T RowGap<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("rowGap", value);

    public static T ColumnGap<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("columnGap", value);

    // ── sizing ───────────────────────────────────────────────────────────────

    /// <summary>Width and height. One argument makes a square.</summary>
    public static T Size<T>(this T node, Dim width, Dim? height = null) where T : ILayoutNode
    {
        node.Props.Set("width", width.ToIr());
        node.Props.Set("height", (height ?? width).ToIr());
        return node;
    }

    public static T Width<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("width", value.ToIr());

    public static T Height<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("height", value.ToIr());

    public static T MinWidth<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("minWidth", value.ToIr());

    public static T MinHeight<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("minHeight", value.ToIr());

    public static T MaxWidth<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("maxWidth", value.ToIr());

    public static T MaxHeight<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("maxHeight", value.ToIr());

    // ── box edges ────────────────────────────────────────────────────────────

    /// <summary>
    /// Border widths, CSS shorthand: one value for all four sides, or up to four.
    /// An omitted side follows CSS — right defaults to top, bottom to top, left
    /// to right — so named arguments such as <c>BorderWidth(top: 2)</c> behave.
    /// </summary>
    public static T BorderWidth<T>(this T node, double top, double? right = null, double? bottom = null, double? left = null)
        where T : ILayoutNode =>
        node.Box(BorderKeys, top, right, bottom, left);

    public static T BorderColor<T>(this T node, string value) where T : ILayoutNode =>
        node.Set("borderColor", value);

    /// <inheritdoc cref="BorderWidth{T}(T,double,double?,double?,double?)"/>
    public static T Margin<T>(this T node, Dim top, Dim? right = null, Dim? bottom = null, Dim? left = null)
        where T : ILayoutNode =>
        node.Box(MarginKeys, top, right, bottom, left);

    public static T MarginTop<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("marginTop", value.ToIr());

    public static T MarginRight<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("marginRight", value.ToIr());

    public static T MarginBottom<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("marginBottom", value.ToIr());

    public static T MarginLeft<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("marginLeft", value.ToIr());

    /// <inheritdoc cref="BorderWidth{T}(T,double,double?,double?,double?)"/>
    public static T Padding<T>(this T node, Dim top, Dim? right = null, Dim? bottom = null, Dim? left = null)
        where T : ILayoutNode =>
        node.Box(PaddingKeys, top, right, bottom, left);

    public static T PaddingTop<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("paddingTop", value.ToIr());

    public static T PaddingRight<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("paddingRight", value.ToIr());

    public static T PaddingBottom<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("paddingBottom", value.ToIr());

    public static T PaddingLeft<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("paddingLeft", value.ToIr());

    // ── insets ───────────────────────────────────────────────────────────────

    public static T Top<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("top", value.ToIr());

    public static T Right<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("right", value.ToIr());

    public static T Bottom<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("bottom", value.ToIr());

    public static T Left<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("left", value.ToIr());

    /// <summary>The leading inset, which flips with the writing direction.</summary>
    public static T Start<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("start", value.ToIr());

    /// <summary>The trailing inset, which flips with the writing direction.</summary>
    public static T End<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("end", value.ToIr());

    public static T Inset<T>(this T node, Dim value) where T : ILayoutNode =>
        node.Set("inset", value.ToIr());

    // ── grid placement ───────────────────────────────────────────────────────

    public static T GridColumn<T>(this T node, int start, int? span = null) where T : ILayoutNode
    {
        node.Props.Set("gridColumnStart", start);
        node.Props.Set("gridColumnSpan", span);
        return node;
    }

    public static T GridRow<T>(this T node, int start, int? span = null) where T : ILayoutNode
    {
        node.Props.Set("gridRowStart", start);
        node.Props.Set("gridRowSpan", span);
        return node;
    }

    // ── pagination ───────────────────────────────────────────────────────────

    /// <summary>Force or forbid a page break at this node. Needs <c>pageHeight</c>.</summary>
    public static T PageBreak<T>(this T node, PageBreakMode value) where T : ILayoutNode =>
        node.Set("pageBreak", value.Value);

    // ── transforms ───────────────────────────────────────────────────────────

    public static T TranslateX<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("translateX", value);

    public static T TranslateY<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("translateY", value);

    /// <summary>Rotation in degrees, about the node's centre.</summary>
    public static T Rotate<T>(this T node, double degrees) where T : ILayoutNode =>
        node.Set("rotation", degrees);

    /// <summary>Scale. One argument scales both axes.</summary>
    public static T Scale<T>(this T node, double x, double? y = null) where T : ILayoutNode =>
        node.Set("scale", new List<object?> { x, y ?? x });

    // ── paint ────────────────────────────────────────────────────────────────

    /// <summary>Add background layers: CSS colours, gradients, or a <c>Photo</c>.</summary>
    public static T Bg<T>(this T node, params Paint[] layers) where T : ILayoutNode =>
        node.Background(layers);

    /// <inheritdoc cref="Bg{T}(T,Paint[])"/>
    public static T Background<T>(this T node, params Paint[] layers) where T : ILayoutNode
    {
        var values = new List<object?>(layers.Length);
        foreach (var layer in layers)
        {
            values.Add(layer.ToIr());
        }
        node.Props.Push("background", values);
        return node;
    }

    public static T Opacity<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("opacity", value);

    /// <summary>Corner radii: one value for all four, or up to four clockwise from the top left.</summary>
    public static T CornerRadius<T>(this T node, params double[] radii) where T : ILayoutNode
    {
        var values = new List<object?>(radii.Length);
        foreach (var radius in radii)
        {
            values.Add(radius);
        }
        return node.Set("cornerRadius", values);
    }

    /// <inheritdoc cref="CornerRadius{T}(T,double[])"/>
    public static T Rounded<T>(this T node, params double[] radii) where T : ILayoutNode =>
        node.CornerRadius(radii);

    /// <inheritdoc cref="CornerRadius{T}(T,double[])"/>
    public static T BorderRadius<T>(this T node, params double[] radii) where T : ILayoutNode =>
        node.CornerRadius(radii);

    /// <summary>Squircle-ness, 0..1. Figma's corner smoothing.</summary>
    public static T CornerSmoothing<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("cornerSmoothing", value);

    /// <inheritdoc cref="CornerSmoothing{T}(T,double)"/>
    public static T BorderSmoothing<T>(this T node, double value) where T : ILayoutNode =>
        node.Set("cornerSmoothing", value);

    public static T Corner<T>(this T node, Corner value) where T : ILayoutNode =>
        node.Set("corner", value.Value);

    /// <summary>Add CSS <c>box-shadow</c> strings.</summary>
    public static T Shadow<T>(this T node, params string[] shadows) where T : ILayoutNode
    {
        node.Props.Push("shadows", shadows);
        return node;
    }

    // ── CSS filters, applied in the order they are added ─────────────────────

    public static T Blur<T>(this T node, double radius) where T : ILayoutNode =>
        node.Filter($"blur({radius.ToIrString()}px)");

    public static T Brightness<T>(this T node, double amount) where T : ILayoutNode =>
        node.Filter($"brightness({amount.ToIrString()})");

    public static T Contrast<T>(this T node, double amount) where T : ILayoutNode =>
        node.Filter($"contrast({amount.ToIrString()})");

    public static T Grayscale<T>(this T node, double amount) where T : ILayoutNode =>
        node.Filter($"grayscale({amount.ToIrString()})");

    public static T HueRotate<T>(this T node, double degrees) where T : ILayoutNode =>
        node.Filter($"hue-rotate({degrees.ToIrString()})");

    public static T Invert<T>(this T node, double amount) where T : ILayoutNode =>
        node.Filter($"invert({amount.ToIrString()})");

    public static T Saturate<T>(this T node, double amount) where T : ILayoutNode =>
        node.Filter($"saturate({amount.ToIrString()})");

    public static T Sepia<T>(this T node, double amount) where T : ILayoutNode =>
        node.Filter($"sepia({amount.ToIrString()})");

    // ── internals ────────────────────────────────────────────────────────────

    private static T Set<T>(this T node, string key, object? value) where T : INode
    {
        node.Props.Set(key, value);
        return node;
    }

    private static T Filter<T>(this T node, string css) where T : ILayoutNode
    {
        node.Props.Push("filters", [css]);
        return node;
    }

    private static T Box<T>(this T node, string[] keys, Dim top, Dim? right, Dim? bottom, Dim? left)
        where T : INode
    {
        if (right is null && bottom is null && left is null)
        {
            node.Props.Set(keys[0], top.ToIr());
            return node;
        }
        node.Props.Set(keys[1], top.ToIr());
        node.Props.Set(keys[2], (right ?? top).ToIr());
        node.Props.Set(keys[3], (bottom ?? top).ToIr());
        node.Props.Set(keys[4], (left ?? right ?? top).ToIr());
        return node;
    }

    private static T Box<T>(this T node, string[] keys, double top, double? right, double? bottom, double? left)
        where T : INode
    {
        if (right is null && bottom is null && left is null)
        {
            node.Props.Set(keys[0], top);
            return node;
        }
        node.Props.Set(keys[1], top);
        node.Props.Set(keys[2], right ?? top);
        node.Props.Set(keys[3], bottom ?? top);
        node.Props.Set(keys[4], left ?? right ?? top);
        return node;
    }

    internal static string ToIrString(this double value) =>
        value.ToString("R", System.Globalization.CultureInfo.InvariantCulture);
}
