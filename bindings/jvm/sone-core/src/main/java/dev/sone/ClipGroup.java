package dev.sone;

/** Clips every child to an SVG path. */
public final class ClipGroup extends Node implements LayoutProps<ClipGroup> {

    public ClipGroup(String clipPath, Node... children) {
        super("clip-group");
        set("clipPath", clipPath);
        adopt(children);
    }
}
