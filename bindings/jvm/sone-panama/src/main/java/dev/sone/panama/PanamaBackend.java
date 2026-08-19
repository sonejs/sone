package dev.sone.panama;

import dev.sone.AssetException;
import dev.sone.Backend;
import dev.sone.Granularity;
import dev.sone.IrException;
import dev.sone.OutputFormat;
import dev.sone.RenderException;
import dev.sone.SoneException;

import java.lang.foreign.Arena;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.ValueLayout;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/**
 * The desktop backend: the C ABI reached through {@code java.lang.foreign}.
 *
 * <p>Chosen by {@code Backends} wherever Panama exists, which is every JVM from
 * 22 on and no Android at all.
 */
public final class PanamaBackend implements Backend {

    private static final MemoryLayout.PathElement DATA = MemoryLayout.PathElement.groupElement("data");
    private static final MemoryLayout.PathElement ITEMS = MemoryLayout.PathElement.groupElement("items");
    private static final MemoryLayout.PathElement LEN = MemoryLayout.PathElement.groupElement("len");

    private static final long BUFFER_DATA = Native.BUFFER.byteOffset(DATA);
    private static final long BUFFER_LEN = Native.BUFFER.byteOffset(LEN);
    private static final long LIST_ITEMS = Native.BUFFER_LIST.byteOffset(ITEMS);
    private static final long LIST_LEN = Native.BUFFER_LIST.byteOffset(LEN);
    private static final long OPTIONS_FORMAT =
            Native.RENDER_OPTIONS.byteOffset(MemoryLayout.PathElement.groupElement("format"));
    private static final long OPTIONS_DENSITY =
            Native.RENDER_OPTIONS.byteOffset(MemoryLayout.PathElement.groupElement("density"));
    private static final long OPTIONS_QUALITY =
            Native.RENDER_OPTIONS.byteOffset(MemoryLayout.PathElement.groupElement("quality"));
    private static final long OPTIONS_STRICT =
            Native.RENDER_OPTIONS.byteOffset(MemoryLayout.PathElement.groupElement("strict"));

    private final MemorySegment handle;
    private boolean closed;

    /** @param baseDir the directory relative asset paths resolve against */
    public PanamaBackend(String baseDir) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment dir = arena.allocateFrom(baseDir == null ? System.getProperty("user.dir") : baseDir);
            handle = (MemorySegment) Native.ENGINE_NEW.invokeExact(dir);
        } catch (Throwable e) {
            throw wrap(e);
        }
        if (handle.equals(MemorySegment.NULL)) {
            throw new SoneException("could not create a sone engine");
        }
    }

    @Override
    public String version() {
        try {
            return Native.string((MemorySegment) Native.VERSION.invokeExact());
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    @Override
    public synchronized void close() {
        if (closed) {
            return;
        }
        closed = true;
        try {
            Native.ENGINE_FREE.invokeExact(handle);
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    // ── fonts and assets ────────────────────────────────────────────────────

    /** Register a font family from raw TTF/OTF bytes. */
    @Override
    public synchronized void registerFont(String name, byte[] data) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment bytes = arena.allocate(Math.max(data.length, 1));
            MemorySegment.copy(data, 0, bytes, ValueLayout.JAVA_BYTE, 0, data.length);
            check((int) Native.REGISTER_FONT.invokeExact(
                    live(), arena.allocateFrom(name), bytes, (long) data.length));
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** Register a font family from a file. */
    @Override
    public synchronized void registerFontFile(String name, String path) {
        try (Arena arena = Arena.ofConfined()) {
            check((int) Native.REGISTER_FONT_FILE.invokeExact(
                    live(), arena.allocateFrom(name), arena.allocateFrom(path)));
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** Make bytes available to documents as {@code asset:name}. */
    @Override
    public synchronized void registerImage(String name, byte[] data) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment bytes = arena.allocate(Math.max(data.length, 1));
            MemorySegment.copy(data, 0, bytes, ValueLayout.JAVA_BYTE, 0, data.length);
            check((int) Native.REGISTER_IMAGE.invokeExact(
                    live(), arena.allocateFrom(name), bytes, (long) data.length));
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** Whether a family has been registered. */
    @Override
    public synchronized boolean hasFont(String name) {
        try (Arena arena = Arena.ofConfined()) {
            return (boolean) Native.HAS_FONT.invokeExact(live(), arena.allocateFrom(name));
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** Every registered family name. */
    @Override
    public synchronized List<String> fontFamilies() {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.BUFFER);
            check((int) Native.FONT_FAMILIES.invokeExact(live(), out));
            String json = new String(take(out), StandardCharsets.UTF_8);
            return dev.sone.Json.readStringArray(json);
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** Drop every registered font. */
    @Override
    public synchronized void resetFonts() {
        try {
            Native.RESET_FONTS.invokeExact(live());
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    // ── rendering ───────────────────────────────────────────────────────────

    /** Render an IR document to bytes. */
    @Override
    public synchronized byte[] render(String document, OutputFormat format, Double density,
            double quality, boolean strict) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.BUFFER);
            check((int) Native.RENDER_JSON.invokeExact(live(), arena.allocateFrom(document),
                    options(arena, format, density, quality, strict), out));
            return take(out);
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** One raster image per page. Requires {@code pageHeight} in the document config. */
    @Override
    public synchronized List<byte[]> renderPages(String document, OutputFormat format, Double density,
            double quality, boolean strict) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment list = arena.allocate(Native.BUFFER_LIST);
            try {
                check((int) Native.RENDER_PAGES.invokeExact(live(), arena.allocateFrom(document),
                        options(arena, format, density, quality, strict), list));
                long count = list.get(ValueLayout.JAVA_LONG, LIST_LEN);
                MemorySegment items = list.get(ValueLayout.ADDRESS, LIST_ITEMS)
                        .reinterpret(count * Native.BUFFER.byteSize());
                List<byte[]> pages = new ArrayList<>((int) count);
                for (long index = 0; index < count; index++) {
                    MemorySegment page = items.asSlice(index * Native.BUFFER.byteSize(), Native.BUFFER);
                    pages.add(read(page));
                }
                return pages;
            } finally {
                Native.BUFFER_LIST_FREE.invokeExact(list);
            }
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** The computed layout tree, as JSON. */
    @Override
    public synchronized String dumpLayout(String document) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.BUFFER);
            check((int) Native.DUMP_LAYOUT.invokeExact(live(), arena.allocateFrom(document), out));
            return new String(take(out), StandardCharsets.UTF_8);
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    /** Dataset-style metadata, as JSON. */
    @Override
    public synchronized String dumpMetadata(String document, Granularity granularity) {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment out = arena.allocate(Native.BUFFER);
            check((int) Native.DUMP_METADATA.invokeExact(live(), arena.allocateFrom(document),
                    arena.allocateFrom(granularity.value()), out));
            return new String(take(out), StandardCharsets.UTF_8);
        } catch (Throwable e) {
            throw wrap(e);
        }
    }

    // ── internals ───────────────────────────────────────────────────────────

    private MemorySegment live() {
        if (closed) {
            throw new IllegalStateException("this engine has been closed");
        }
        return handle;
    }

    private static MemorySegment options(Arena arena, OutputFormat format, Double density,
            double quality, boolean strict) {
        MemorySegment options = arena.allocate(Native.RENDER_OPTIONS);
        options.set(ValueLayout.JAVA_INT, OPTIONS_FORMAT, format.code());
        // Zero tells the engine to fall back to the document's own config.
        options.set(ValueLayout.JAVA_FLOAT, OPTIONS_DENSITY, density == null ? 0f : density.floatValue());
        options.set(ValueLayout.JAVA_FLOAT, OPTIONS_QUALITY, (float) quality);
        options.set(ValueLayout.JAVA_INT, OPTIONS_STRICT, strict ? 1 : 0);
        return options;
    }

    /** Copies a buffer out and releases it. */
    private static byte[] take(MemorySegment buffer) throws Throwable {
        try {
            return read(buffer);
        } finally {
            Native.BUFFER_FREE.invokeExact(buffer);
        }
    }

    private static byte[] read(MemorySegment buffer) {
        long length = buffer.get(ValueLayout.JAVA_LONG, BUFFER_LEN);
        MemorySegment data = buffer.get(ValueLayout.ADDRESS, BUFFER_DATA);
        if (length == 0 || data.equals(MemorySegment.NULL)) {
            return new byte[0];
        }
        return data.reinterpret(length).toArray(ValueLayout.JAVA_BYTE);
    }

    private void check(int status) throws Throwable {
        if (status == Native.OK) {
            return;
        }
        String message = Native.string((MemorySegment) Native.LAST_ERROR.invokeExact(handle));
        if (message == null) {
            message = "sone failed with status " + status;
        }
        throw switch (status) {
            case Native.INVALID_ARGUMENT -> new IllegalArgumentException(message);
            case Native.IR_ERROR -> new IrException(message);
            case Native.ASSET_ERROR -> new AssetException(message);
            default -> new RenderException(message);
        };
    }

    private static RuntimeException wrap(Throwable e) {
        if (e instanceof RuntimeException runtime) {
            return runtime;
        }
        if (e instanceof Error error) {
            throw error;
        }
        return new SoneException(e.toString());
    }
}
