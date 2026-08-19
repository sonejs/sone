package dev.sone;

/** The paragraph's base direction for bidi resolution. */
public enum BaseDir {
    LTR("ltr"),
    RTL("rtl"),
    AUTO("auto");

    private final String value;

    BaseDir(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
