package dev.sone;

/** How wrapped flex lines are distributed on the cross axis. */
public enum AlignContent {
    FLEX_START("flex-start"),
    FLEX_END("flex-end"),
    CENTER("center"),
    STRETCH("stretch"),
    SPACE_BETWEEN("space-between"),
    SPACE_AROUND("space-around"),
    SPACE_EVENLY("space-evenly");

    private final String value;

    AlignContent(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
