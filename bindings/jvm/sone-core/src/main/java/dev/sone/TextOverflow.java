package dev.sone;

/** What a clipped paragraph ends with. */
public enum TextOverflow {
    CLIP("clip"),
    ELLIPSIS("ellipsis");

    private final String value;

    TextOverflow(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
