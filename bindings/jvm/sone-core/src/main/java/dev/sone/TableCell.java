package dev.sone;

/** A table cell. */
public final class TableCell extends Node implements LayoutProps<TableCell> {

    public TableCell(Node... children) {
        super("table-cell");
        adopt(children);
    }

    public TableCell colspan(int value) {
        return set("colspan", value);
    }

    public TableCell rowspan(int value) {
        return set("rowspan", value);
    }
}
