package dev.sone;

/** How a node participates in layout. */
public enum Display {
    NONE("none"),
    FLEX("flex"),
    CONTENTS("contents");

    private final String value;

    Display(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
