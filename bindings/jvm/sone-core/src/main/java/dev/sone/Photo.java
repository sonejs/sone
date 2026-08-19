package dev.sone;

import java.util.Base64;

/** An image. */
public final class Photo extends Node implements LayoutProps<Photo> {

    /** From a path, a URL, or {@code asset:name}. */
    public Photo(String src) {
        super("photo");
        set("src", src);
    }

    /** From raw bytes, inlined into the document as a data URL. */
    public static Photo of(byte[] data) {
        return new Photo("data:application/octet-stream;base64," + Base64.getEncoder().encodeToString(data));
    }

    /** How the image fills its box. */
    public Photo scaleType(ScaleType value) {
        return set("scaleType", value.value());
    }

    /** The alignment is 0..1 — 0 is start, 0.5 centre, 1 end. */
    public Photo scaleType(ScaleType value, double alignment) {
        return set("scaleType", value.value()).set("scaleAlignment", alignment);
    }

    public Photo preserveAspectRatio() { return set("preserveAspectRatio", true); }
    public Photo preserveAspectRatio(boolean value) { return set("preserveAspectRatio", value); }
    public Photo flipHorizontal() { return set("flipHorizontal", true); }
    public Photo flipVertical() { return set("flipVertical", true); }

    /** The letterbox colour behind a {@code contain} image. */
    public Photo fill(String color) { return set("fill", color); }

    /** An SVG path the image is clipped to. */
    public Photo clipPath(String path) { return set("clipPath", path); }
}
