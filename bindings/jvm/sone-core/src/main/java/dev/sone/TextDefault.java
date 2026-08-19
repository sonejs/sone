package dev.sone;

/** Cascades text styling onto its descendants without drawing a box. */
public final class TextDefault extends Node
        implements SpanStyleProps<TextDefault>, TextBlockProps<TextDefault> {

    public TextDefault(Node... children) {
        super("text-default");
        adopt(children);
    }
}
