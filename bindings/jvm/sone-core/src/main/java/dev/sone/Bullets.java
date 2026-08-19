package dev.sone;

/** A bulleted or numbered list. Named {@code Bullets} because {@code List} is {@code java.util.List}. */
public final class Bullets extends Node implements LayoutProps<Bullets> {

    public Bullets(Node... items) {
        super("list");
        adopt(items);
    }

    /** {@code disc}, {@code circle}, {@code square}, {@code decimal}, {@code dash}, {@code none}, or literal text. */
    public Bullets listStyle(String value) { return set("listStyle", value); }

    /** A styled marker node. {@code {}} in its text is replaced with the item number. */
    public Bullets listStyle(Node marker) { return set("listStyle", marker); }

    public Bullets markerGap(double value) { return set("markerGap", value); }
    public Bullets markerOffset(double value) { return set("markerOffset", value); }
    public Bullets startIndex(int value) { return set("startIndex", value); }
}
