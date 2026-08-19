package dev.sone;

/** Main-axis distribution. */
public enum JustifyContent {
    FLEX_START("flex-start"),
    FLEX_END("flex-end"),
    CENTER("center"),
    SPACE_BETWEEN("space-between"),
    SPACE_AROUND("space-around"),
    SPACE_EVENLY("space-evenly");

    private final String value;

    JustifyContent(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
