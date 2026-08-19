package dev.sone;

import java.util.Locale;

/**
 * A length: a number, {@code auto}, or a percentage.
 *
 * <p>Java has no implicit user-defined conversions, so this is the one place the
 * {@code number | "auto" | "%"} union of the IR costs a call: {@code width(50)}
 * takes the number overload, {@code width(Dim.percent(50))} the object one.
 */
public final class Dim {

    /** Size from content, the CSS {@code auto} keyword. */
    public static final Dim AUTO = new Dim(null, "auto");

    private final Double number;
    private final String text;

    private Dim(Double number, String text) {
        this.number = number;
        this.text = text;
    }

    public static Dim of(double value) {
        return new Dim(value, null);
    }

    /** A percentage of the containing block. */
    public static Dim percent(double value) {
        return new Dim(null, trim(value) + "%");
    }

    /** Parses {@code "auto"}, {@code "50%"} or a bare number. */
    public static Dim parse(String value) {
        String trimmed = value.trim();
        if (trimmed.equals("auto")) {
            return AUTO;
        }
        if (trimmed.endsWith("%")) {
            Double.parseDouble(trimmed.substring(0, trimmed.length() - 1));
            return new Dim(null, trimmed);
        }
        return of(Double.parseDouble(trimmed));
    }

    /** The value as the IR carries it: a number, or a string. */
    Object toIr() {
        return text != null ? text : number;
    }

    private static String trim(double value) {
        if (value == Math.rint(value) && !Double.isInfinite(value)) {
            return String.valueOf((long) value);
        }
        return String.format(Locale.ROOT, "%s", value);
    }

    @Override
    public String toString() {
        return text != null ? text : trim(number);
    }

    @Override
    public boolean equals(Object other) {
        if (!(other instanceof Dim dim)) {
            return false;
        }
        return java.util.Objects.equals(number, dim.number) && java.util.Objects.equals(text, dim.text);
    }

    @Override
    public int hashCode() {
        return java.util.Objects.hash(number, text);
    }
}
