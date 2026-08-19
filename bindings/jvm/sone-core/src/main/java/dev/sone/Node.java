package dev.sone;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * A node in the document tree.
 *
 * <p>Properties live on self-typed interfaces with default methods rather than a
 * superclass: Java has single inheritance, and {@link Text} has to be a box, a
 * styled run and a paragraph at once.
 */
public abstract class Node {

    private final String type;
    private final Map<String, Object> props = new LinkedHashMap<>();
    private final List<Node> children = new ArrayList<>();
    private final List<Object> inline = new ArrayList<>();

    protected Node(String type) {
        this.type = type;
    }

    /** The IR node type, e.g. {@code "column"}. */
    public final String type() {
        return type;
    }

    /** Properties set on this node, in the order they were set. */
    public final Map<String, Object> props() {
        return props;
    }

    /** Container children. Empty for {@code text} and {@code span}. */
    public final List<Node> children() {
        return children;
    }

    /** Paragraph content: strings and {@link Span}s. Only text and span use it. */
    public final List<Object> inline() {
        return inline;
    }

    /** This node as IR JSON. */
    public final String toJson() {
        StringBuilder out = new StringBuilder();
        writeJson(out);
        return out.toString();
    }

    final void writeJson(StringBuilder out) {
        out.append("{\"type\":");
        Json.write(out, type);
        if (!props.isEmpty()) {
            out.append(",\"props\":");
            Json.write(out, props);
        }
        if (!children.isEmpty()) {
            out.append(",\"children\":");
            Json.write(out, children);
        }
        if (!inline.isEmpty()) {
            out.append(",\"inline\":");
            Json.write(out, inline);
        }
        out.append('}');
    }

    /** Adds children, dropping nulls so {@code flag ? badge() : null} works. */
    protected final void adopt(Node... items) {
        for (Node item : items) {
            if (item != null) {
                children.add(item);
            }
        }
    }

    @Override
    public String toString() {
        Object tag = props.get("tag");
        return "<sone:" + type + (tag == null ? "" : " \"" + tag + "\"")
                + " props=" + props.size() + " children=" + children.size() + ">";
    }
}
