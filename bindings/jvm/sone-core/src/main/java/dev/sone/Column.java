package dev.sone;

/** Column node. */
public final class Column extends Node implements LayoutProps<Column> {

    public Column(Node... children) {
        super("column");
        adopt(children);
    }
}
