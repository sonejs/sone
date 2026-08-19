package dev.sone;

/** Writing direction. */
public enum Direction {
    LTR("ltr"),
    RTL("rtl");

    private final String value;

    Direction(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
