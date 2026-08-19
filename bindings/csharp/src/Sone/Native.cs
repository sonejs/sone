using System.Runtime.InteropServices;

namespace Sone;

internal enum SoneStatus
{
    Ok = 0,
    InvalidArgument = 1,
    IrError = 2,
    AssetError = 3,
    RenderError = 4,
}

/// <summary>The output formats the engine can encode.</summary>
public enum OutputFormat
{
    Png = 0,
    Jpeg = 1,
    Webp = 2,

    /// <summary>Raw RGBA pixels, row-major, unpremultiplied.</summary>
    Raw = 3,

    /// <summary>A PDF. With <c>pageHeight</c> set, one page per break and selectable text.</summary>
    Pdf = 4,
    Svg = 5,
}

[StructLayout(LayoutKind.Sequential)]
internal struct SoneRenderOptions
{
    public OutputFormat Format;
    public float Density;
    public float Quality;
    public int Strict;
}

[StructLayout(LayoutKind.Sequential)]
internal struct SoneBuffer
{
    public nint Data;
    public nuint Len;
    public nuint Capacity;
}

[StructLayout(LayoutKind.Sequential)]
internal struct SoneBufferList
{
    public nint Items;
    public nuint Len;
    public nuint Capacity;
}

/// <summary>
/// The C ABI from <c>include/sone.h</c>, one declaration per function.
/// </summary>
/// <remarks>
/// <c>LibraryImport</c> rather than <c>DllImport</c>: the marshalling is
/// source-generated, so the assembly stays trimmable and NativeAOT-clean. The
/// document is passed as a NUL-terminated UTF-8 pointer we build ourselves,
/// which keeps the IR from round-tripping through UTF-16 on the way out.
/// </remarks>
internal static unsafe partial class Native
{
    internal const string Library = "sone";

    /// <summary>
    /// Installs the resolver before the first P/Invoke. A static constructor
    /// rather than a module initializer: the runtime already guarantees this
    /// runs before any member of this class is touched, and a library has no
    /// business running code at assembly load.
    /// </summary>
    static Native() => NativeLibrary.SetDllImportResolver(typeof(Native).Assembly, NativeLoader.Resolve);

    [LibraryImport(Library, EntryPoint = "sone_engine_new", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial nint EngineNew(string? baseDir);

    [LibraryImport(Library, EntryPoint = "sone_engine_free")]
    internal static partial void EngineFree(nint engine);

    [LibraryImport(Library, EntryPoint = "sone_engine_last_error")]
    internal static partial nint EngineLastError(nint engine);

    [LibraryImport(Library, EntryPoint = "sone_register_font", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial SoneStatus RegisterFont(nint engine, string name, byte* data, nuint len);

    [LibraryImport(Library, EntryPoint = "sone_register_font_file", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial SoneStatus RegisterFontFile(nint engine, string name, string path);

    [LibraryImport(Library, EntryPoint = "sone_register_image", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial SoneStatus RegisterImage(nint engine, string name, byte* data, nuint len);

    [LibraryImport(Library, EntryPoint = "sone_has_font", StringMarshalling = StringMarshalling.Utf8)]
    [return: MarshalAs(UnmanagedType.U1)]
    internal static partial bool HasFont(nint engine, string name);

    [LibraryImport(Library, EntryPoint = "sone_font_families")]
    internal static partial SoneStatus FontFamilies(nint engine, out SoneBuffer buffer);

    [LibraryImport(Library, EntryPoint = "sone_reset_fonts")]
    internal static partial void ResetFonts(nint engine);

    [LibraryImport(Library, EntryPoint = "sone_render_json")]
    internal static partial SoneStatus RenderJson(nint engine, byte* json, SoneRenderOptions options, out SoneBuffer buffer);

    [LibraryImport(Library, EntryPoint = "sone_render_pages")]
    internal static partial SoneStatus RenderPages(nint engine, byte* json, SoneRenderOptions options, out SoneBufferList list);

    [LibraryImport(Library, EntryPoint = "sone_dump_layout")]
    internal static partial SoneStatus DumpLayout(nint engine, byte* json, out SoneBuffer buffer);

    [LibraryImport(Library, EntryPoint = "sone_dump_metadata", StringMarshalling = StringMarshalling.Utf8)]
    internal static partial SoneStatus DumpMetadata(nint engine, byte* json, string granularity, out SoneBuffer buffer);

    [LibraryImport(Library, EntryPoint = "sone_buffer_free")]
    internal static partial void BufferFree(ref SoneBuffer buffer);

    [LibraryImport(Library, EntryPoint = "sone_buffer_list_free")]
    internal static partial void BufferListFree(ref SoneBufferList list);

    [LibraryImport(Library, EntryPoint = "sone_version")]
    internal static partial nint Version();
}

/// <summary>
/// Finds the native library. NuGet puts it in <c>runtimes/{rid}/native</c> and
/// the default resolver finds it there; this exists for the case NuGet does not
/// cover — running against a <c>cargo build</c> in a checkout.
/// </summary>
internal static class NativeLoader
{
    /// <summary>A full path to the library, or a directory containing it.</summary>
    internal const string PathVariable = "SONE_NATIVE_LIBRARY";

    internal static nint Resolve(string name, System.Reflection.Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (name != Native.Library)
        {
            return nint.Zero;
        }

        var hint = Environment.GetEnvironmentVariable(PathVariable);
        if (!string.IsNullOrEmpty(hint))
        {
            var candidate = Directory.Exists(hint) ? Path.Combine(hint, FileName) : hint;
            if (File.Exists(candidate) && NativeLibrary.TryLoad(candidate, out var handle))
            {
                return handle;
            }
            throw new DllNotFoundException(
                $"{PathVariable} is set to \"{hint}\" but no loadable {FileName} is there");
        }

        // Zero hands the request back to the default resolver.
        return nint.Zero;
    }

    private static string FileName =>
        RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? "sone.dll"
        : RuntimeInformation.IsOSPlatform(OSPlatform.OSX) ? "libsone.dylib"
        : "libsone.so";
}
