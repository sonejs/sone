package dev.sone;

/** TableRow node. */
public final class TableRow extends Node implements LayoutProps<TableRow> {

    public TableRow(Node... children) {
        super("table-row");
        adopt(children);
    }
}
