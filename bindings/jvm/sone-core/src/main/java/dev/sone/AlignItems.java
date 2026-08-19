package dev.sone;

/** Cross-axis alignment. Also the type of {@code alignSelf}. */
public enum AlignItems {
    FLEX_START("flex-start"),
    FLEX_END("flex-end"),
    CENTER("center"),
    STRETCH("stretch"),
    BASELINE("baseline");

    private final String value;

    AlignItems(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
