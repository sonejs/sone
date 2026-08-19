package dev.sone;

/** The entry point. */
public final class Sone {

    private Sone() {
    }

    /**
     * Wrap a node with render configuration.
     *
     * <pre>{@code
     * Sone.render(root).density(2).save(Path.of("card.png"));
     * }</pre>
     */
    public static Rendering render(Node root) {
        return new Rendering(root);
    }

    /** An explicit page break. Only meaningful with a page height set. */
    public static Column pageBreak() {
        return new Column().height(0).pageBreak(PageBreakMode.BEFORE);
    }
}
