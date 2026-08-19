package dev.sone;

import java.util.List;

/**
 * Owns the font registry and the decoded-image cache.
 *
 * <p>A facade over whichever {@link Backend} the platform supplies. Skia's font
 * collection is shared inside an engine, so one engine renders one document at
 * a time and every call here is synchronized. Give each thread its own
 * {@code Engine} for real parallelism rather than sharing one.
 */
public final class Engine implements AutoCloseable {

    private static Engine defaultEngine;

    private final Backend backend;

    /** @param baseDir the directory relative asset paths resolve against */
    public Engine(String baseDir) {
        this.backend = Backends.create(baseDir);
    }

    public Engine() {
        this(null);
    }

    /** The process-wide engine, used when no explicit one is passed. */
    public static synchronized Engine getDefault() {
        if (defaultEngine == null) {
            defaultEngine = new Engine();
        }
        return defaultEngine;
    }

    /** The native library version. */
    public static String version() {
        try (Engine engine = new Engine()) {
            return engine.backend.version();
        }
    }

    /** Which backend is in use — {@code dev.sone.panama…} or {@code dev.sone.jna…}. */
    public String backendName() {
        return backend.getClass().getName();
    }

    @Override
    public synchronized void close() {
        backend.close();
    }

    // ── fonts and assets ────────────────────────────────────────────────────

    /** Register a font family from raw TTF/OTF bytes. */
    public synchronized void registerFont(String name, byte[] data) {
        backend.registerFont(name, data);
    }

    /** Register a font family from a file. */
    public synchronized void registerFontFile(String name, String path) {
        backend.registerFontFile(name, path);
    }

    /** Make bytes available to documents as {@code asset:name}. */
    public synchronized void registerImage(String name, byte[] data) {
        backend.registerImage(name, data);
    }

    /** Whether a family has been registered. */
    public synchronized boolean hasFont(String name) {
        return backend.hasFont(name);
    }

    /** Every registered family name. */
    public synchronized List<String> fontFamilies() {
        return backend.fontFamilies();
    }

    /** Drop every registered font. */
    public synchronized void resetFonts() {
        backend.resetFonts();
    }

    // ── rendering ───────────────────────────────────────────────────────────

    /** Render an IR document to bytes. */
    public synchronized byte[] render(String document, OutputFormat format, Double density,
            double quality, boolean strict) {
        return backend.render(document, format, density, quality, strict);
    }

    public byte[] render(String document, OutputFormat format) {
        return render(document, format, null, 1.0, false);
    }

    /** One raster image per page. Requires {@code pageHeight} in the document config. */
    public synchronized List<byte[]> renderPages(String document, OutputFormat format, Double density,
            double quality, boolean strict) {
        return backend.renderPages(document, format, density, quality, strict);
    }

    /** The computed layout tree, as JSON. */
    public synchronized String dumpLayout(String document) {
        return backend.dumpLayout(document);
    }

    /** Dataset-style metadata, as JSON. */
    public synchronized String dumpMetadata(String document, Granularity granularity) {
        return backend.dumpMetadata(document, granularity);
    }
}
