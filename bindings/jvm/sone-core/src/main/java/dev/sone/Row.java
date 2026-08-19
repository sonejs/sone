package dev.sone;

/** Row node. */
public final class Row extends Node implements LayoutProps<Row> {

    public Row(Node... children) {
        super("row");
        adopt(children);
    }
}
