package dev.sone;

/** How two path segments meet. */
public enum StrokeJoin {
    BEVEL("bevel"),
    MITER("miter"),
    ROUND("round");

    private final String value;

    StrokeJoin(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
