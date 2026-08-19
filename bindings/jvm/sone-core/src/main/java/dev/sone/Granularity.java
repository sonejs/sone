package dev.sone;

/** The granularity of the boxes {@code metadata} returns. */
public enum Granularity {
    NODE("node"),
    LINE("line"),
    WORD("word");

    private final String value;

    Granularity(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
