package dev.sone.jna;

import com.sun.jna.Library;
import com.sun.jna.Memory;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import dev.sone.AssetException;
import dev.sone.Backend;
import dev.sone.Granularity;
import dev.sone.IrException;
import dev.sone.Json;
import dev.sone.LibraryPath;
import dev.sone.OutputFormat;
import dev.sone.RenderException;
import dev.sone.SoneException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/**
 * The Android backend: the same C ABI, reached through JNA.
 *
 * <p>Android has no {@code java.lang.foreign} — ART does not implement Panama,
 * and it cannot be desugared because it needs VM support — so the choice is JNA
 * or a hand-written JNI shim. JNA wins here because it reuses
 * {@code include/sone.h} unchanged, which keeps the project's one-C-ABI
 * promise: a JNI layer would have been a second ABI to keep in step.
 *
 * <p>Its per-call overhead is irrelevant at this granularity. One render is one
 * call carrying a whole document; the marshalling cost sits next to a Skia
 * rasterization.
 *
 * <p>This backend is not Android-only — it runs anywhere, and the desktop test
 * suite exercises it, which is how the Android path stays honest without a
 * device in the loop.
 */
public final class JnaBackend implements Backend {

    /** The C ABI. Names and signatures come straight from {@code include/sone.h}. */
    public interface Lib extends Library {
        Pointer sone_engine_new(String baseDir);

        void sone_engine_free(Pointer engine);

        Pointer sone_engine_last_error(Pointer engine);

        int sone_register_font(Pointer engine, String name, Pointer data, long len);

        int sone_register_font_file(Pointer engine, String name, String path);

        int sone_register_image(Pointer engine, String name, Pointer data, long len);

        boolean sone_has_font(Pointer engine, String name);

        int sone_font_families(Pointer engine, SoneBuffer out);

        void sone_reset_fonts(Pointer engine);

        int sone_render_json(Pointer engine, String json, SoneRenderOptions.ByValue options, SoneBuffer out);

        int sone_render_pages(Pointer engine, String json, SoneRenderOptions.ByValue options, SoneBufferList out);

        int sone_dump_layout(Pointer engine, String json, SoneBuffer out);

        int sone_dump_metadata(Pointer engine, String json, String granularity, SoneBuffer out);

        void sone_buffer_free(SoneBuffer buffer);

        void sone_buffer_list_free(SoneBufferList list);

        Pointer sone_version();
    }

    /**
     * {@code uintptr_t} is declared as {@code long}, which makes this 64-bit
     * only. That matches what ships: rust-skia publishes no 32-bit Android
     * binary, and Play Store has required 64-bit since 2019.
     */
    @Structure.FieldOrder({"data", "len", "capacity"})
    public static class SoneBuffer extends Structure {
        public Pointer data;
        public long len;
        public long capacity;

        public SoneBuffer() {
        }

        /** Reads a buffer the library already allocated, such as one page of many. */
        public SoneBuffer(Pointer at) {
            super(at);
            read();
        }

        byte[] bytes() {
            if (data == null || len == 0) {
                return new byte[0];
            }
            return data.getByteArray(0, (int) len);
        }
    }

    @Structure.FieldOrder({"items", "len", "capacity"})
    public static class SoneBufferList extends Structure {
        public Pointer items;
        public long len;
        public long capacity;
    }

    @Structure.FieldOrder({"format", "density", "quality", "strict"})
    public static class SoneRenderOptions extends Structure {
        public int format;
        public float density;
        public float quality;
        public int strict;

        public static class ByValue extends SoneRenderOptions implements Structure.ByValue {
        }
    }

    private static final int OK = 0;
    private static final int INVALID_ARGUMENT = 1;
    private static final int IR_ERROR = 2;
    private static final int ASSET_ERROR = 3;

    private static final Lib LIB = Native.load(LibraryPath.locate(), Lib.class);

    private final Pointer handle;
    private boolean closed;

    public JnaBackend(String baseDir) {
        handle = LIB.sone_engine_new(baseDir != null ? baseDir : System.getProperty("user.dir", "."));
        if (handle == null) {
            throw new SoneException("could not create a sone engine");
        }
    }

    @Override
    public String version() {
        Pointer pointer = LIB.sone_version();
        return pointer == null ? "unknown" : pointer.getString(0);
    }

    @Override
    public synchronized void close() {
        if (!closed) {
            closed = true;
            LIB.sone_engine_free(handle);
        }
    }

    // ── fonts and assets ────────────────────────────────────────────────────

    @Override
    public synchronized void registerFont(String name, byte[] data) {
        try (Memory buffer = bytes(data)) {
            check(LIB.sone_register_font(live(), name, buffer, data.length));
        }
    }

    @Override
    public synchronized void registerFontFile(String name, String path) {
        check(LIB.sone_register_font_file(live(), name, path));
    }

    @Override
    public synchronized void registerImage(String name, byte[] data) {
        try (Memory buffer = bytes(data)) {
            check(LIB.sone_register_image(live(), name, buffer, data.length));
        }
    }

    @Override
    public synchronized boolean hasFont(String name) {
        return LIB.sone_has_font(live(), name);
    }

    @Override
    public synchronized List<String> fontFamilies() {
        SoneBuffer out = new SoneBuffer();
        try {
            check(LIB.sone_font_families(live(), out));
            out.read();
            return Json.readStringArray(new String(out.bytes(), StandardCharsets.UTF_8));
        } finally {
            LIB.sone_buffer_free(out);
        }
    }

    @Override
    public synchronized void resetFonts() {
        LIB.sone_reset_fonts(live());
    }

    // ── rendering ───────────────────────────────────────────────────────────

    @Override
    public synchronized byte[] render(String document, OutputFormat format, Double density,
            double quality, boolean strict) {
        SoneBuffer out = new SoneBuffer();
        try {
            check(LIB.sone_render_json(live(), document, options(format, density, quality, strict), out));
            out.read();
            return out.bytes();
        } finally {
            LIB.sone_buffer_free(out);
        }
    }

    @Override
    public synchronized List<byte[]> renderPages(String document, OutputFormat format, Double density,
            double quality, boolean strict) {
        SoneBufferList list = new SoneBufferList();
        try {
            check(LIB.sone_render_pages(live(), document, options(format, density, quality, strict), list));
            list.read();
            List<byte[]> pages = new ArrayList<>((int) list.len);
            if (list.items != null && list.len > 0) {
                int stride = new SoneBuffer().size();
                for (int index = 0; index < list.len; index++) {
                    pages.add(new SoneBuffer(list.items.share((long) index * stride)).bytes());
                }
            }
            return pages;
        } finally {
            LIB.sone_buffer_list_free(list);
        }
    }

    @Override
    public synchronized String dumpLayout(String document) {
        SoneBuffer out = new SoneBuffer();
        try {
            check(LIB.sone_dump_layout(live(), document, out));
            out.read();
            return new String(out.bytes(), StandardCharsets.UTF_8);
        } finally {
            LIB.sone_buffer_free(out);
        }
    }

    @Override
    public synchronized String dumpMetadata(String document, Granularity granularity) {
        SoneBuffer out = new SoneBuffer();
        try {
            check(LIB.sone_dump_metadata(live(), document, granularity.value(), out));
            out.read();
            return new String(out.bytes(), StandardCharsets.UTF_8);
        } finally {
            LIB.sone_buffer_free(out);
        }
    }

    // ── internals ───────────────────────────────────────────────────────────

    private Pointer live() {
        if (closed) {
            throw new IllegalStateException("this engine has been closed");
        }
        return handle;
    }

    private static Memory bytes(byte[] data) {
        Memory buffer = new Memory(Math.max(data.length, 1));
        buffer.write(0, data, 0, data.length);
        return buffer;
    }

    private static SoneRenderOptions.ByValue options(OutputFormat format, Double density,
            double quality, boolean strict) {
        SoneRenderOptions.ByValue options = new SoneRenderOptions.ByValue();
        options.format = format.code();
        // Zero tells the engine to fall back to the document's own config.
        options.density = density == null ? 0f : density.floatValue();
        options.quality = (float) quality;
        options.strict = strict ? 1 : 0;
        return options;
    }

    private void check(int status) {
        if (status == OK) {
            return;
        }
        Pointer pointer = LIB.sone_engine_last_error(handle);
        String message = pointer == null
                ? "sone failed with status " + status
                : pointer.getString(0);
        switch (status) {
            case INVALID_ARGUMENT:
                throw new IllegalArgumentException(message);
            case IR_ERROR:
                throw new IrException(message);
            case ASSET_ERROR:
                throw new AssetException(message);
            default:
                throw new RenderException(message);
        }
    }
}
