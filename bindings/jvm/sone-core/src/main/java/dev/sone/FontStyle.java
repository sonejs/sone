package dev.sone;

/** Roman or slanted. */
public enum FontStyle {
    NORMAL("normal"),
    ITALIC("italic"),
    OBLIQUE("oblique");

    private final String value;

    FontStyle(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
