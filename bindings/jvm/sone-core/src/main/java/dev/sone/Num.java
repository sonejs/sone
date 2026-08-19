package dev.sone;

/** Formats a double the way CSS wants it: no trailing {@code .0}. */
final class Num {

    private Num() {
    }

    static String of(double value) {
        if (value == Math.rint(value) && !Double.isInfinite(value)) {
            return String.valueOf((long) value);
        }
        return String.valueOf(value);
    }
}
