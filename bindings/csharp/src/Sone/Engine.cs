using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace Sone;

/// <summary>
/// Owns the font registry and the decoded-image cache.
/// </summary>
/// <remarks>
/// Skia's font collection is shared inside an engine, so one engine renders one
/// document at a time — every call here takes a lock. For real parallelism give
/// each thread its own <see cref="Engine"/> rather than sharing one.
/// </remarks>
public sealed class Engine : IDisposable
{
    private static readonly Lazy<Engine> Shared = new(() => new Engine(Environment.CurrentDirectory));

    private readonly object _gate = new();
    private nint _handle;

    /// <param name="baseDir">
    /// The directory relative asset paths resolve against. Null means the
    /// process working directory.
    /// </param>
    public Engine(string? baseDir = null)
    {
        _handle = Native.EngineNew(baseDir);
        if (_handle == nint.Zero)
        {
            throw new SoneException("could not create a sone engine");
        }
    }

    ~Engine() => Free();

    /// <summary>The process-wide engine used when no explicit one is passed.</summary>
    public static Engine Default => Shared.Value;

    /// <summary>The native library version.</summary>
    public static string Version => Marshal.PtrToStringUTF8(Native.Version()) ?? "unknown";

    // ── fonts and assets ─────────────────────────────────────────────────────

    /// <summary>Register a font family from raw TTF/OTF bytes.</summary>
    public unsafe void RegisterFont(string name, ReadOnlySpan<byte> data)
    {
        ArgumentNullException.ThrowIfNull(name);
        lock (_gate)
        {
            fixed (byte* pointer = data)
            {
                Check(Native.RegisterFont(Live, name, pointer, (nuint)data.Length));
            }
        }
    }

    /// <summary>Register a font family from a file.</summary>
    public void RegisterFontFile(string name, string path)
    {
        ArgumentNullException.ThrowIfNull(name);
        ArgumentNullException.ThrowIfNull(path);
        lock (_gate)
        {
            Check(Native.RegisterFontFile(Live, name, path));
        }
    }

    /// <summary>Make bytes available to documents as <c>asset:name</c>.</summary>
    public unsafe void RegisterImage(string name, ReadOnlySpan<byte> data)
    {
        ArgumentNullException.ThrowIfNull(name);
        lock (_gate)
        {
            fixed (byte* pointer = data)
            {
                Check(Native.RegisterImage(Live, name, pointer, (nuint)data.Length));
            }
        }
    }

    /// <summary>Whether a family has been registered.</summary>
    public bool HasFont(string name)
    {
        ArgumentNullException.ThrowIfNull(name);
        lock (_gate)
        {
            return Native.HasFont(Live, name);
        }
    }

    /// <summary>Every registered family name.</summary>
    public IReadOnlyList<string> FontFamilies()
    {
        byte[] json;
        lock (_gate)
        {
            SoneBuffer buffer = default;
            var status = Native.FontFamilies(Live, out buffer);
            json = Collect(status, ref buffer);
        }
        return JsonSerializer.Deserialize(json, SoneJson.Default.StringArray) ?? [];
    }

    /// <summary>Drop every registered font.</summary>
    public void ResetFonts()
    {
        lock (_gate)
        {
            Native.ResetFonts(Live);
        }
    }

    // ── rendering ────────────────────────────────────────────────────────────

    /// <summary>Render an IR document to bytes.</summary>
    public byte[] Render(
        string document,
        OutputFormat format = OutputFormat.Png,
        double? density = null,
        double quality = 1.0,
        bool strict = false) =>
        Render(Utf8(document), format, density, quality, strict);

    /// <summary>One raster image per page. Requires <c>pageHeight</c> in the document config.</summary>
    public IReadOnlyList<byte[]> RenderPages(
        string document,
        OutputFormat format = OutputFormat.Png,
        double? density = null,
        double quality = 1.0,
        bool strict = false) =>
        RenderPages(Utf8(document), format, density, quality, strict);

    /// <summary>The computed layout tree, as JSON.</summary>
    public string DumpLayout(string document) => DumpLayout(Utf8(document));

    /// <summary>Dataset-style metadata, as JSON.</summary>
    public string DumpMetadata(string document, string granularity = "node") =>
        DumpMetadata(Utf8(document), granularity);

    // The overloads Rendering uses: the document is already NUL-terminated
    // UTF-8, so nothing re-encodes on the way to the engine.

    internal unsafe byte[] Render(byte[] document, OutputFormat format, double? density, double quality, bool strict)
    {
        var options = OptionsFor(format, density, quality, strict);
        lock (_gate)
        {
            fixed (byte* json = document)
            {
                SoneBuffer buffer = default;
                var status = Native.RenderJson(Live, json, in options, out buffer);
                return Collect(status, ref buffer);
            }
        }
    }

    internal unsafe IReadOnlyList<byte[]> RenderPages(byte[] document, OutputFormat format, double? density, double quality, bool strict)
    {
        var options = OptionsFor(format, density, quality, strict);
        lock (_gate)
        {
            fixed (byte* json = document)
            {
                SoneBufferList list = default;
                var status = Native.RenderPages(Live, json, in options, out list);
                try
                {
                    Check(status);
                    var items = (SoneBuffer*)list.Items;
                    var pages = new byte[(int)list.Len][];
                    for (var i = 0; i < pages.Length; i++)
                    {
                        pages[i] = Copy(items[i]);
                    }
                    return pages;
                }
                finally
                {
                    Native.BufferListFree(ref list);
                }
            }
        }
    }

    internal unsafe string DumpLayout(byte[] document)
    {
        lock (_gate)
        {
            fixed (byte* json = document)
            {
                SoneBuffer buffer = default;
                var status = Native.DumpLayout(Live, json, out buffer);
                return Encoding.UTF8.GetString(Collect(status, ref buffer));
            }
        }
    }

    internal unsafe string DumpMetadata(byte[] document, string granularity)
    {
        lock (_gate)
        {
            fixed (byte* json = document)
            {
                SoneBuffer buffer = default;
                var status = Native.DumpMetadata(Live, json, granularity, out buffer);
                return Encoding.UTF8.GetString(Collect(status, ref buffer));
            }
        }
    }

    // ── lifetime ─────────────────────────────────────────────────────────────

    public void Dispose()
    {
        Free();
        GC.SuppressFinalize(this);
    }

    private void Free()
    {
        lock (_gate)
        {
            if (_handle != nint.Zero)
            {
                Native.EngineFree(_handle);
                _handle = nint.Zero;
            }
        }
    }

    // ── internals ────────────────────────────────────────────────────────────

    /// <summary>The handle, or a clear error rather than a segfault.</summary>
    private nint Live
    {
        get
        {
            ObjectDisposedException.ThrowIf(_handle == nint.Zero, this);
            return _handle;
        }
    }

    private static SoneRenderOptions OptionsFor(OutputFormat format, double? density, double quality, bool strict) => new()
    {
        Format = format,
        // Zero tells the engine to fall back to the document's own config.
        Density = (float)(density ?? 0.0),
        Quality = (float)quality,
        Strict = strict ? 1 : 0,
    };

    /// <summary>A NUL-terminated UTF-8 copy, which is what the C ABI takes.</summary>
    internal static byte[] Utf8(string document)
    {
        var bytes = new byte[Encoding.UTF8.GetByteCount(document) + 1];
        Encoding.UTF8.GetBytes(document, bytes);
        return bytes;
    }

    private byte[] Collect(SoneStatus status, ref SoneBuffer buffer)
    {
        try
        {
            Check(status);
            return Copy(buffer);
        }
        finally
        {
            Native.BufferFree(ref buffer);
        }
    }

    private static byte[] Copy(SoneBuffer buffer)
    {
        if (buffer.Data == nint.Zero || buffer.Len == 0)
        {
            return [];
        }
        var bytes = new byte[(int)buffer.Len];
        Marshal.Copy(buffer.Data, bytes, 0, bytes.Length);
        return bytes;
    }

    private void Check(SoneStatus status)
    {
        if (status == SoneStatus.Ok)
        {
            return;
        }
        var message = Marshal.PtrToStringUTF8(Native.EngineLastError(_handle))
                      ?? $"sone failed with {status}";
        throw status switch
        {
            SoneStatus.InvalidArgument => new ArgumentException(message),
            SoneStatus.IrError => new IrException(message),
            SoneStatus.AssetError => new AssetException(message),
            _ => (Exception)new RenderException(message),
        };
    }
}
