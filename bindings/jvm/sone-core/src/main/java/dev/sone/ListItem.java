package dev.sone;

/** One item in a {@link Bullets} list. */
public final class ListItem extends Node implements LayoutProps<ListItem> {

    public ListItem(Node... children) {
        super("list-item");
        adopt(children);
    }

    /** Override the list's marker for this item alone. */
    public ListItem marker(Node value) {
        return set("marker", value);
    }
}
