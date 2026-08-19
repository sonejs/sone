package dev.sone;

/** The line-breaking algorithm. */
public enum LineBreakMode {
    GREEDY("greedy"),
    KNUTH_PLASS("knuth-plass");

    private final String value;

    LineBreakMode(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
