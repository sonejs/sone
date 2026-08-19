package dev.sone;

/** Span-level text styling. */
public interface SpanStyleProps<SELF extends SpanStyleProps<SELF>> extends NodeProps<SELF> {

    default SELF color(String value) { return set("color", value); }

    /**
     * The font size, not the box size.
     *
     * <p>{@link Text} inherits this and {@link LayoutProps#size(double)} with the
     * same signature, so the compiler makes it override — which is exactly where
     * the rule that {@code Text.size} is the font size gets written down.
     */
    default SELF size(double value) { return set("size", value); }

    /** The font stack, in fallback order. */
    default SELF font(String... families) { return set("font", java.util.List.of(families)); }

    default SELF style(FontStyle value) { return set("style", value.value()); }

    /** A CSS keyword such as {@code "bold"}. */
    default SELF weight(String value) { return set("weight", value); }

    /** A numeric weight, 100..900. */
    default SELF weight(double value) { return set("weight", value); }

    default SELF letterSpacing(double value) { return set("letterSpacing", value); }
    default SELF wordSpacing(double value) { return set("wordSpacing", value); }

    default SELF underline() { return underline(1.0); }
    default SELF underline(double thickness) { return set("underline", thickness); }
    default SELF overline() { return overline(1.0); }
    default SELF overline(double thickness) { return set("overline", thickness); }
    default SELF lineThrough() { return lineThrough(1.0); }
    default SELF lineThrough(double thickness) { return set("lineThrough", thickness); }

    /** The no-argument form is an explicit null: "use the text colour". */
    default SELF underlineColor() { return setNullable("underlineColor", null); }
    default SELF underlineColor(String value) { return setNullable("underlineColor", value); }
    default SELF overlineColor() { return setNullable("overlineColor", null); }
    default SELF overlineColor(String value) { return setNullable("overlineColor", value); }
    default SELF lineThroughColor() { return setNullable("lineThroughColor", null); }
    default SELF lineThroughColor(String value) { return setNullable("lineThroughColor", value); }
    default SELF highlight() { return setNullable("highlightColor", null); }
    default SELF highlight(String value) { return setNullable("highlightColor", value); }

    /** Add CSS {@code text-shadow} strings. */
    default SELF dropShadow(String... shadows) { return push("dropShadows", (Object[]) shadows); }

    /** The glyph outline colour. */
    default SELF strokeColor(String value) { return set("strokeColor", value); }

    /** The glyph outline width. */
    default SELF strokeWidth(double value) { return set("strokeWidth", value); }

    /** Shift the run off its baseline — superscripts, subscripts. */
    default SELF offsetY(double value) { return set("offsetY", value); }

    /** Force this run's direction, overriding bidi resolution. */
    default SELF textDir(Direction value) { return set("textDir", value.value()); }
}
