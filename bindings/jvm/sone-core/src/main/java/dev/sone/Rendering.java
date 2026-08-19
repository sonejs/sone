package dev.sone;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * A node plus its render configuration, with one method per output format.
 *
 * <p>Built with {@link Sone#render(Node)}: Java has no named arguments, so the
 * configuration is set by chaining rather than passed in one call.
 */
public final class Rendering {

    /** A font the document carries with it, so the CLI renders it identically. */
    public record FontSource(String name, String src) {
    }

    private final Node root;
    private final Map<String, Object> config = new LinkedHashMap<>();
    private final List<FontSource> fonts = new ArrayList<>();
    private Engine engine;
    private String cached;

    Rendering(Node root) {
        this.root = root;
    }

    // ── configuration ───────────────────────────────────────────────────────

    public Rendering engine(Engine value) {
        this.engine = value;
        return this;
    }

    public Rendering width(double value) {
        return put("width", value);
    }

    public Rendering height(double value) {
        return put("height", value);
    }

    /** A CSS colour painted behind everything. */
    public Rendering background(String value) {
        return put("background", value);
    }

    /** Raster scale factor. A render-time density overrides this. */
    public Rendering density(double value) {
        return put("density", value);
    }

    /** Turn the document into pages of this height. */
    public Rendering pageHeight(double value) {
        return put("pageHeight", value);
    }

    public Rendering margin(double all) {
        return put("margin", all);
    }

    public Rendering margin(double top, double right, double bottom, double left) {
        return put("margin", new LinkedHashMap<>(Map.of()) {{
            put("top", top);
            put("right", right);
            put("bottom", bottom);
            put("left", left);
        }});
    }

    public Rendering lastPageHeight(LastPageHeight value) {
        return put("lastPageHeight", value.value());
    }

    /**
     * Drawn at the top of every page. Use the literal tokens
     * {@code {pageNumber}} and {@code {totalPages}} — the engine substitutes them.
     */
    public Rendering header(Node node) {
        return put("header", node);
    }

    /** Drawn at the bottom of every page. */
    public Rendering footer(Node node) {
        return put("footer", node);
    }

    public Rendering font(String name, String src) {
        fonts.add(new FontSource(name, src));
        cached = null;
        return this;
    }

    // ── the document ────────────────────────────────────────────────────────

    /** The IR document as JSON, built once and reused. */
    public String toJson() {
        if (cached != null) {
            return cached;
        }
        StringBuilder out = new StringBuilder();
        out.append("{\"sone\":1");
        if (!fonts.isEmpty()) {
            out.append(",\"fonts\":[");
            for (int i = 0; i < fonts.size(); i++) {
                if (i > 0) {
                    out.append(',');
                }
                Map<String, Object> entry = new LinkedHashMap<>();
                entry.put("name", fonts.get(i).name());
                entry.put("src", fonts.get(i).src());
                Json.write(out, entry);
            }
            out.append(']');
        }
        if (!config.isEmpty()) {
            out.append(",\"config\":");
            Json.write(out, config);
        }
        out.append(",\"root\":");
        root.writeJson(out);
        out.append('}');
        return cached = out.toString();
    }

    // ── outputs ─────────────────────────────────────────────────────────────

    public byte[] png() {
        return png(null);
    }

    public byte[] png(Double density) {
        return engine().render(toJson(), OutputFormat.PNG, density, 1.0, false);
    }

    public byte[] jpeg(double quality) {
        return engine().render(toJson(), OutputFormat.JPEG, null, quality, false);
    }

    public byte[] webp(double quality) {
        return engine().render(toJson(), OutputFormat.WEBP, null, quality, false);
    }

    /** Raw RGBA pixels, row-major, unpremultiplied. */
    public byte[] raw() {
        return raw(null);
    }

    public byte[] raw(Double density) {
        return engine().render(toJson(), OutputFormat.RAW, density, 1.0, false);
    }

    /** A PDF. With a page height set, one page per break and selectable text. */
    public byte[] pdf() {
        return engine().render(toJson(), OutputFormat.PDF);
    }

    public byte[] svg() {
        return engine().render(toJson(), OutputFormat.SVG);
    }

    /** One raster image per page. Requires a page height. */
    public List<byte[]> pages() {
        return pages(OutputFormat.PNG, null);
    }

    public List<byte[]> pages(OutputFormat format, Double density) {
        return engine().renderPages(toJson(), format, density, 1.0, false);
    }

    /** Render and write to {@code path}, inferring the format from its extension. */
    public Path save(Path path) {
        return save(path, null);
    }

    public Path save(Path path, Double density) {
        byte[] bytes = switch (formatFor(path.toString())) {
            case PNG -> png(density);
            case JPEG -> jpeg(1.0);
            case WEBP -> webp(1.0);
            case RAW -> raw(density);
            case PDF -> pdf();
            case SVG -> svg();
        };
        try {
            Files.write(path, bytes);
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
        return path;
    }

    /** Write {@code name-1.png}, {@code name-2.png}, … next to {@code path}. */
    public List<Path> savePages(Path path) {
        String name = path.getFileName().toString();
        int dot = name.lastIndexOf('.');
        String stem = dot < 0 ? name : name.substring(0, dot);
        String extension = dot < 0 ? ".png" : name.substring(dot);
        OutputFormat format = dot < 0 ? OutputFormat.PNG : formatFor(name);

        List<byte[]> rendered = pages(format, null);
        List<Path> written = new ArrayList<>(rendered.size());
        for (int index = 0; index < rendered.size(); index++) {
            Path target = path.resolveSibling(stem + "-" + (index + 1) + extension);
            try {
                Files.write(target, rendered.get(index));
            } catch (IOException e) {
                throw new UncheckedIOException(e);
            }
            written.add(target);
        }
        return written;
    }

    // ── introspection ───────────────────────────────────────────────────────

    /**
     * The computed layout tree, as JSON.
     *
     * <p>A string rather than a parsed tree, so this binding does not hand every
     * consumer a JSON library version to reconcile.
     */
    public String layoutJson() {
        return engine().dumpLayout(toJson());
    }

    /** Dataset-style boxes at node, line or word granularity, as JSON. */
    public String metadataJson() {
        return metadataJson(Granularity.NODE);
    }

    public String metadataJson(Granularity granularity) {
        return engine().dumpMetadata(toJson(), granularity);
    }

    private Engine engine() {
        return engine != null ? engine : Engine.getDefault();
    }

    private Rendering put(String key, Object value) {
        config.put(key, value);
        cached = null;
        return this;
    }

    private static OutputFormat formatFor(String path) {
        int dot = path.lastIndexOf('.');
        String extension = dot < 0 ? "" : path.substring(dot + 1).toLowerCase(Locale.ROOT);
        return switch (extension) {
            case "png" -> OutputFormat.PNG;
            case "jpg", "jpeg" -> OutputFormat.JPEG;
            case "webp" -> OutputFormat.WEBP;
            case "pdf" -> OutputFormat.PDF;
            case "svg" -> OutputFormat.SVG;
            case "raw", "rgba" -> OutputFormat.RAW;
            default -> throw new IllegalArgumentException("cannot infer an output format from " + path);
        };
    }
}
