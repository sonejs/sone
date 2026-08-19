package dev.sone;

/** Horizontal alignment inside a paragraph. */
public enum TextAlign {
    LEFT("left"),
    RIGHT("right"),
    CENTER("center"),
    JUSTIFY("justify");

    private final String value;

    TextAlign(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
