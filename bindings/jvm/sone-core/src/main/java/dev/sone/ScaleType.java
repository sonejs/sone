package dev.sone;

/** How a photo fills its box. */
public enum ScaleType {
    COVER("cover"),
    FILL("fill"),
    CONTAIN("contain");

    private final String value;

    ScaleType(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
