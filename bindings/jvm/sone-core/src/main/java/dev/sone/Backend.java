package dev.sone;

import java.util.List;

/**
 * What an engine implementation has to provide.
 *
 * <p>Two exist, and which one loads is a platform fact rather than a choice.
 * Desktop JVMs get {@code dev.sone.panama.PanamaBackend}, which calls the C ABI
 * through {@code java.lang.foreign}. Android has no such package — ART does not
 * implement Panama, and it cannot be desugared because it needs VM support — so
 * Android gets {@code dev.sone.jna.JnaBackend}, which reaches the same C ABI
 * through JNA.
 *
 * <p>Everything above this interface is shared: the whole builder, the property
 * interfaces, the JSON writer and {@link Rendering} never learn which one is
 * underneath.
 */
public interface Backend extends AutoCloseable {

    void registerFont(String name, byte[] data);

    void registerFontFile(String name, String path);

    void registerImage(String name, byte[] data);

    boolean hasFont(String name);

    List<String> fontFamilies();

    void resetFonts();

    byte[] render(String document, OutputFormat format, Double density, double quality, boolean strict);

    List<byte[]> renderPages(String document, OutputFormat format, Double density, double quality, boolean strict);

    String dumpLayout(String document);

    String dumpMetadata(String document, Granularity granularity);

    /** The native library version. */
    String version();

    @Override
    void close();
}
