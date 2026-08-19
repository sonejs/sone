package dev.sone;

/** The main axis of a container. */
public enum FlexDirection {
    ROW("row"),
    COLUMN("column"),
    ROW_REVERSE("row-reverse"),
    COLUMN_REVERSE("column-reverse");

    private final String value;

    FlexDirection(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
