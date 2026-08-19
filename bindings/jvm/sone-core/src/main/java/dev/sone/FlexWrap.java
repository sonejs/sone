package dev.sone;

/** Whether flex items wrap onto new lines. */
public enum FlexWrap {
    WRAP("wrap"),
    NOWRAP("nowrap"),
    WRAP_REVERSE("wrap-reverse");

    private final String value;

    FlexWrap(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
