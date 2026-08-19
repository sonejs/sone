package dev.sone;

/**
 * A paragraph. Both a box and a run of text.
 *
 * <p>{@link LayoutProps} and {@link SpanStyleProps} both declare
 * {@code size(double)}, so Java requires this class to override it — which is a
 * convenient place to put the rule that {@code Text.size} is the font size, the
 * way the TypeScript API omits the layout {@code size} from a text builder. Use
 * {@link #width(double)} and {@link #height(double)} for the box.
 */
public final class Text extends Node
        implements LayoutProps<Text>, SpanStyleProps<Text>, TextBlockProps<Text> {

    public Text(Object... content) {
        super("text");
        for (Object item : content) {
            if (item instanceof Span || item instanceof String) {
                inline().add(item);
            } else if (item != null) {
                throw new IllegalArgumentException("text content is a String or a Span, got " + item.getClass());
            }
        }
    }

    /** The font size. */
    @Override
    public Text size(double value) {
        return set("size", value);
    }

    /** Append content after construction. */
    public Text content(Object... items) {
        for (Object item : items) {
            inline().add(item);
        }
        return this;
    }
}
