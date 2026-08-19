package dev.sone;

/** How an open path ends. */
public enum StrokeCap {
    BUTT("butt"),
    ROUND("round"),
    SQUARE("square");

    private final String value;

    StrokeCap(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
