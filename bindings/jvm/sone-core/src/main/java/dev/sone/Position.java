package dev.sone;

/** Positioning scheme. */
public enum Position {
    ABSOLUTE("absolute"),
    RELATIVE("relative"),
    STATIC("static");

    private final String value;

    Position(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
