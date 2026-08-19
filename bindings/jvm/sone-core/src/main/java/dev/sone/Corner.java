package dev.sone;

/** The shape a corner radius produces. */
public enum Corner {
    CUT("cut"),
    ROUND("round");

    private final String value;

    Corner(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
