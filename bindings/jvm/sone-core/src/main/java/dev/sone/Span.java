package dev.sone;

/** A styled run inside a {@link Text}. */
public final class Span extends Node implements SpanStyleProps<Span> {

    public Span(String text) {
        super("span");
        if (text != null) {
            inline().add(text);
        }
    }
}
