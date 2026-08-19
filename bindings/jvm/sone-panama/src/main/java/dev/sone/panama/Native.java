package dev.sone.panama;

import dev.sone.LibraryPath;
import dev.sone.SoneException;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemoryLayout;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.StructLayout;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;

/**
 * The C ABI from {@code include/sone.h}. Nothing above this class sees a
 * {@link MemorySegment}.
 *
 * <p>Hand-written Panama rather than {@code jextract} output: sixteen functions
 * and three structs is less code than the generator's plumbing, and it keeps the
 * build to a plain {@code mvn package} with no extra tool on the path.
 */
public final class Native {

    static final int OK = 0;
    static final int INVALID_ARGUMENT = 1;
    static final int IR_ERROR = 2;
    static final int ASSET_ERROR = 3;
    static final int RENDER_ERROR = 4;

    /** {@code uintptr_t} is pointer-width, which every supported target makes 64-bit. */
    static final StructLayout BUFFER = MemoryLayout.structLayout(
            ValueLayout.ADDRESS.withName("data"),
            ValueLayout.JAVA_LONG.withName("len"),
            ValueLayout.JAVA_LONG.withName("capacity"));

    static final StructLayout BUFFER_LIST = MemoryLayout.structLayout(
            ValueLayout.ADDRESS.withName("items"),
            ValueLayout.JAVA_LONG.withName("len"),
            ValueLayout.JAVA_LONG.withName("capacity"));

    static final StructLayout RENDER_OPTIONS = MemoryLayout.structLayout(
            ValueLayout.JAVA_INT.withName("format"),
            ValueLayout.JAVA_FLOAT.withName("density"),
            ValueLayout.JAVA_FLOAT.withName("quality"),
            ValueLayout.JAVA_INT.withName("strict"));

    static final MethodHandle ENGINE_NEW;
    static final MethodHandle ENGINE_FREE;
    static final MethodHandle LAST_ERROR;
    static final MethodHandle REGISTER_FONT;
    static final MethodHandle REGISTER_FONT_FILE;
    static final MethodHandle REGISTER_IMAGE;
    static final MethodHandle HAS_FONT;
    static final MethodHandle FONT_FAMILIES;
    static final MethodHandle RESET_FONTS;
    static final MethodHandle RENDER_JSON;
    static final MethodHandle RENDER_PAGES;
    static final MethodHandle DUMP_LAYOUT;
    static final MethodHandle DUMP_METADATA;
    static final MethodHandle BUFFER_FREE;
    static final MethodHandle BUFFER_LIST_FREE;
    static final MethodHandle VERSION;

    static {
        Linker linker = Linker.nativeLinker();
        SymbolLookup lookup =
                SymbolLookup.libraryLookup(java.nio.file.Path.of(LibraryPath.locate()), Arena.global());

        ValueLayout.OfInt i32 = ValueLayout.JAVA_INT;
        ValueLayout.OfLong i64 = ValueLayout.JAVA_LONG;
        java.lang.foreign.AddressLayout ptr = ValueLayout.ADDRESS;

        ENGINE_NEW = handle(linker, lookup, "sone_engine_new", FunctionDescriptor.of(ptr, ptr));
        ENGINE_FREE = handle(linker, lookup, "sone_engine_free", FunctionDescriptor.ofVoid(ptr));
        LAST_ERROR = handle(linker, lookup, "sone_engine_last_error", FunctionDescriptor.of(ptr, ptr));
        REGISTER_FONT = handle(linker, lookup, "sone_register_font",
                FunctionDescriptor.of(i32, ptr, ptr, ptr, i64));
        REGISTER_FONT_FILE = handle(linker, lookup, "sone_register_font_file",
                FunctionDescriptor.of(i32, ptr, ptr, ptr));
        REGISTER_IMAGE = handle(linker, lookup, "sone_register_image",
                FunctionDescriptor.of(i32, ptr, ptr, ptr, i64));
        HAS_FONT = handle(linker, lookup, "sone_has_font",
                FunctionDescriptor.of(ValueLayout.JAVA_BOOLEAN, ptr, ptr));
        FONT_FAMILIES = handle(linker, lookup, "sone_font_families",
                FunctionDescriptor.of(i32, ptr, ptr));
        RESET_FONTS = handle(linker, lookup, "sone_reset_fonts", FunctionDescriptor.ofVoid(ptr));
        // Options by pointer, not by value: struct-by-value is the one part of
        // a C ABI that FFI layers disagree about.
        RENDER_JSON = handle(linker, lookup, "sone_render_json",
                FunctionDescriptor.of(i32, ptr, ptr, ptr, ptr));
        RENDER_PAGES = handle(linker, lookup, "sone_render_pages",
                FunctionDescriptor.of(i32, ptr, ptr, ptr, ptr));
        DUMP_LAYOUT = handle(linker, lookup, "sone_dump_layout",
                FunctionDescriptor.of(i32, ptr, ptr, ptr));
        DUMP_METADATA = handle(linker, lookup, "sone_dump_metadata",
                FunctionDescriptor.of(i32, ptr, ptr, ptr, ptr));
        BUFFER_FREE = handle(linker, lookup, "sone_buffer_free", FunctionDescriptor.ofVoid(ptr));
        BUFFER_LIST_FREE = handle(linker, lookup, "sone_buffer_list_free", FunctionDescriptor.ofVoid(ptr));
        VERSION = handle(linker, lookup, "sone_version", FunctionDescriptor.of(ptr));
    }

    private Native() {
    }

    private static MethodHandle handle(Linker linker, SymbolLookup lookup, String name,
            FunctionDescriptor descriptor) {
        return linker.downcallHandle(
                lookup.find(name).orElseThrow(() -> new SoneException("missing symbol " + name)),
                descriptor);
    }

    /** Reads a NUL-terminated C string that the library owns. */
    static String string(MemorySegment pointer) {
        if (pointer == null || pointer.equals(MemorySegment.NULL)) {
            return null;
        }
        return pointer.reinterpret(Long.MAX_VALUE).getString(0);
    }
}
