namespace Sone;

/// <summary>
/// Span-level text styling, shared by <see cref="TextNode"/>,
/// <see cref="SpanNode"/> and <see cref="TextDefaultNode"/>.
/// </summary>
public static class SpanProps
{
    public static T Color<T>(this T node, string value) where T : ISpanStyle =>
        node.Set("color", value);

    /// <summary>
    /// The font size. <see cref="TextNode"/> declares this as an instance method
    /// so it wins over the layout <c>Size</c>, matching the TypeScript API.
    /// </summary>
    public static T Size<T>(this T node, double value) where T : ISpanStyle =>
        node.Set("size", value);

    /// <summary>The font stack, in fallback order.</summary>
    public static T Font<T>(this T node, params string[] families) where T : ISpanStyle
    {
        var values = new List<object?>(families.Length);
        foreach (var family in families)
        {
            values.Add(family);
        }
        return node.Set("font", values);
    }

    public static T Style<T>(this T node, FontStyle value) where T : ISpanStyle =>
        node.Set("style", value.Value);

    public static T Weight<T>(this T node, FontWeight value) where T : ISpanStyle =>
        node.Set("weight", value.ToIr());

    public static T LetterSpacing<T>(this T node, double value) where T : ISpanStyle =>
        node.Set("letterSpacing", value);

    public static T WordSpacing<T>(this T node, double value) where T : ISpanStyle =>
        node.Set("wordSpacing", value);

    // ── decorations ──────────────────────────────────────────────────────────
    //
    // The colour setters take an explicit null to mean "use the text colour",
    // which the engine distinguishes from the property being absent — so they
    // go through SetNullable rather than Set.

    /// <summary>Underline thickness, as a multiple of the default.</summary>
    public static T Underline<T>(this T node, double thickness = 1.0) where T : ISpanStyle =>
        node.Set("underline", thickness);

    public static T UnderlineColor<T>(this T node, string? value = null) where T : ISpanStyle
    {
        node.Props.SetNullable("underlineColor", value);
        return node;
    }

    public static T Overline<T>(this T node, double thickness = 1.0) where T : ISpanStyle =>
        node.Set("overline", thickness);

    public static T OverlineColor<T>(this T node, string? value = null) where T : ISpanStyle
    {
        node.Props.SetNullable("overlineColor", value);
        return node;
    }

    public static T LineThrough<T>(this T node, double thickness = 1.0) where T : ISpanStyle =>
        node.Set("lineThrough", thickness);

    public static T LineThroughColor<T>(this T node, string? value = null) where T : ISpanStyle
    {
        node.Props.SetNullable("lineThroughColor", value);
        return node;
    }

    /// <summary>A highlight behind the run.</summary>
    public static T Highlight<T>(this T node, string? value = null) where T : ISpanStyle
    {
        node.Props.SetNullable("highlightColor", value);
        return node;
    }

    /// <summary>Add CSS <c>text-shadow</c> strings.</summary>
    public static T DropShadow<T>(this T node, params string[] shadows) where T : ISpanStyle
    {
        node.Props.Push("dropShadows", shadows);
        return node;
    }

    /// <summary>The glyph outline colour.</summary>
    public static T StrokeColor<T>(this T node, string value) where T : ISpanStyle =>
        node.Set("strokeColor", value);

    /// <summary>The glyph outline width.</summary>
    public static T StrokeWidth<T>(this T node, double value) where T : ISpanStyle =>
        node.Set("strokeWidth", value);

    /// <summary>Shift the run off its baseline — superscripts, subscripts.</summary>
    public static T OffsetY<T>(this T node, double value) where T : ISpanStyle =>
        node.Set("offsetY", value);

    /// <summary>Force this run's direction, overriding bidi resolution.</summary>
    public static T TextDir<T>(this T node, Direction value) where T : ISpanStyle =>
        node.Set("textDir", value.Value);

    private static T Set<T>(this T node, string key, object? value) where T : INode
    {
        node.Props.Set(key, value);
        return node;
    }
}

/// <summary>
/// Paragraph-level properties, shared by <see cref="TextNode"/> and
/// <see cref="TextDefaultNode"/>.
/// </summary>
public static class TextBlockProps
{
    /// <summary>Never wrap this paragraph.</summary>
    public static T Nowrap<T>(this T node) where T : ITextBlock =>
        node.Set("nowrap", true);

    /// <summary>
    /// Whether the paragraph wraps. <see cref="TextNode"/> declares this as an
    /// instance method so it wins over the flexbox <c>Wrap</c>.
    /// </summary>
    public static T Wrap<T>(this T node, bool wrap = true) where T : ITextBlock =>
        node.Set("nowrap", !wrap);

    /// <summary>Truncate after this many lines.</summary>
    public static T MaxLines<T>(this T node, double value) where T : ITextBlock =>
        node.Set("maxLines", value);

    public static T LineBreak<T>(this T node, LineBreakMode value) where T : ITextBlock =>
        node.Set("lineBreak", value.Value);

    public static T TextOverflow<T>(this T node, TextOverflow value) where T : ITextBlock =>
        node.Set("textOverflow", value.Value);

    /// <summary>Line height as a multiple of the font size.</summary>
    public static T LineHeight<T>(this T node, double value) where T : ITextBlock =>
        node.Set("lineHeight", value);

    public static T Align<T>(this T node, TextAlign value) where T : ITextBlock =>
        node.Set("align", value.Value);

    /// <summary>First-line indent.</summary>
    public static T Indent<T>(this T node, double value) where T : ITextBlock =>
        node.Set("indentSize", value);

    /// <summary>Indent every line but the first.</summary>
    public static T HangingIndent<T>(this T node, double value) where T : ITextBlock =>
        node.Set("hangingIndentSize", value);

    public static T TabStops<T>(this T node, params double[] stops) where T : ITextBlock
    {
        var values = new List<object?>(stops.Length);
        foreach (var stop in stops)
        {
            values.Add(stop);
        }
        return node.Set("tabStops", values);
    }

    /// <summary>The character filling the space a tab skips.</summary>
    public static T TabLeader<T>(this T node, string value) where T : ITextBlock =>
        node.Set("tabLeader", value);

    /// <summary>Shrink the text until it fits its box.</summary>
    public static T Autofit<T>(this T node, bool value = true) where T : ITextBlock =>
        node.Set("autofit", value);

    /// <summary>Rotation of the text inside its box, in degrees.</summary>
    public static T Orientation<T>(this T node, int degrees) where T : ITextBlock =>
        node.Set("orientation", degrees);

    /// <summary>Paint the glyphs with an image instead of a colour.</summary>
    public static T ClipImage<T>(this T node, INode photo) where T : ITextBlock =>
        node.Set("clipImage", photo);

    /// <summary>The base direction used to resolve bidi runs.</summary>
    public static T BaseDir<T>(this T node, BaseDir value) where T : ITextBlock =>
        node.Set("baseDir", value.Value);

    /// <summary>Greedy wrapping, or balancing for a ragged edge.</summary>
    public static T TextWrap<T>(this T node, TextWrap value) where T : ITextBlock =>
        node.Set("textWrap", value.Value);

    private static T Set<T>(this T node, string key, object? value) where T : INode
    {
        node.Props.Set(key, value);
        return node;
    }
}
