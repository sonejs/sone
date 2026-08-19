package dev.sone;

/** Greedy wrapping, or ragged-edge balancing. */
public enum TextWrap {
    WRAP("wrap"),
    BALANCE("balance");

    private final String value;

    TextWrap(String value) {
        this.value = value;
    }

    public String value() {
        return value;
    }
}
