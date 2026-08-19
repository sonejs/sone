package dev.sone;

/** A grid track: a fixed size, {@code auto}, or an {@code fr} share. */
public final class Track {

    /** A track sized to its content. */
    public static final Track AUTO = new Track("auto");

    private final Object value;

    private Track(Object value) {
        this.value = value;
    }

    public static Track of(double size) {
        return new Track(size);
    }

    /** A share of the free space, the CSS {@code fr} unit. */
    public static Track fr(double value) {
        String text = value == Math.rint(value) ? String.valueOf((long) value) : String.valueOf(value);
        return new Track(text + "fr");
    }

    Object toIr() {
        return value;
    }

    @Override
    public String toString() {
        return String.valueOf(value);
    }
}
