package dev.sone;

/** Whether the final page is full height or shrinks to its content. */
public enum LastPageHeight {
    UNIFORM("uniform"),
    CONTENT("content");

    private final String value;

    LastPageHeight(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
