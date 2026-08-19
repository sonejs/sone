package dev.sone;

import java.util.List;

/** A table. Children are {@link TableRow}s. */
public final class Table extends Node implements LayoutProps<Table> {

    public Table(Node... rows) {
        super("table");
        adopt(rows);
    }

    /** Row and column spacing. One argument sets both. */
    public Table spacing(double all) {
        return spacing(all, all);
    }

    public Table spacing(double row, double column) {
        return set("spacing", List.of(row, column));
    }
}
