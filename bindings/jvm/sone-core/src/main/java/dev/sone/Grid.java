package dev.sone;

import java.util.ArrayList;
import java.util.List;

/** A grid container with row-major auto placement. */
public final class Grid extends Node implements LayoutProps<Grid> {

    public Grid(Node... children) {
        super("grid");
        adopt(children);
    }

    public Grid columns(Track... tracks) { return set("columns", list(tracks)); }
    public Grid rows(Track... tracks) { return set("rows", list(tracks)); }
    public Grid autoRows(Track... tracks) { return set("autoRows", list(tracks)); }
    public Grid autoColumns(Track... tracks) { return set("autoColumns", list(tracks)); }

    private static List<Object> list(Track... tracks) {
        List<Object> values = new ArrayList<>(tracks.length);
        for (Track track : tracks) {
            values.add(track);
        }
        return values;
    }
}
