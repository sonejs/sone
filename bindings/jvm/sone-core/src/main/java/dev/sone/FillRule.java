package dev.sone;

/** Which regions of a self-intersecting path are inside it. */
public enum FillRule {
    EVENODD("evenodd"),
    NONZERO("nonzero");

    private final String value;

    FillRule(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
