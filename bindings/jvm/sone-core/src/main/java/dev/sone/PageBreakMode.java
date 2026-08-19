package dev.sone;

/** Where a page break may or must fall. */
public enum PageBreakMode {
    BEFORE("before"),
    AFTER("after"),
    AVOID("avoid");

    private final String value;

    PageBreakMode(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
