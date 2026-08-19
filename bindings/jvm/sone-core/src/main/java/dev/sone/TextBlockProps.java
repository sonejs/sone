package dev.sone;

/** Paragraph-level properties. */
public interface TextBlockProps<SELF extends TextBlockProps<SELF>> extends NodeProps<SELF> {

    default SELF nowrap() { return set("nowrap", true); }

    /** Whether the paragraph wraps. Not the flexbox {@code wrap}. */
    default SELF wrap(boolean value) { return set("nowrap", !value); }

    default SELF maxLines(double value) { return set("maxLines", value); }
    default SELF lineBreak(LineBreakMode value) { return set("lineBreak", value.value()); }
    default SELF textOverflow(TextOverflow value) { return set("textOverflow", value.value()); }
    default SELF lineHeight(double value) { return set("lineHeight", value); }
    default SELF align(TextAlign value) { return set("align", value.value()); }
    default SELF indent(double value) { return set("indentSize", value); }
    default SELF hangingIndent(double value) { return set("hangingIndentSize", value); }

    default SELF tabStops(double... stops) {
        java.util.List<Double> values = new java.util.ArrayList<>(stops.length);
        for (double stop : stops) {
            values.add(stop);
        }
        return set("tabStops", values);
    }

    default SELF tabLeader(String value) { return set("tabLeader", value); }
    default SELF autofit() { return set("autofit", true); }
    default SELF autofit(boolean value) { return set("autofit", value); }

    /** Rotation of the text inside its box, in degrees. */
    default SELF orientation(int degrees) { return set("orientation", degrees); }

    /** Paint the glyphs with an image instead of a colour. */
    default SELF clipImage(Photo photo) { return set("clipImage", photo); }

    /** The base direction used to resolve bidi runs. */
    default SELF baseDir(BaseDir value) { return set("baseDir", value.value()); }

    /** Greedy wrapping, or balancing for a ragged edge. */
    default SELF textWrap(TextWrap value) { return set("textWrap", value.value()); }
}
