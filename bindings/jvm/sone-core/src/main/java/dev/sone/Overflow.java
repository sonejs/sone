package dev.sone;

/** What happens to content past a node's box. */
public enum Overflow {
    VISIBLE("visible"),
    HIDDEN("hidden"),
    SCROLL("scroll");

    private final String value;

    Overflow(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
