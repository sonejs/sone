namespace Sone;

/// <summary>
/// Font registration on the process-wide engine, for scripts that do not want
/// to create one. Everything here is <see cref="Engine.Default"/>.
/// </summary>
/// <remarks>
/// Skia carries no system fonts, so at least one family must be registered
/// before rendering any text.
/// </remarks>
public static class Font
{
    /// <summary>Register a family from a TTF/OTF file.</summary>
    public static void Load(string name, string path) =>
        Engine.Default.RegisterFontFile(name, path);

    /// <summary>Register a family from raw TTF/OTF bytes.</summary>
    public static void Load(string name, ReadOnlySpan<byte> data) =>
        Engine.Default.RegisterFont(name, data);

    /// <summary>Whether a family has been registered.</summary>
    public static bool Has(string name) => Engine.Default.HasFont(name);

    /// <summary>Every registered family name.</summary>
    public static IReadOnlyList<string> Families() => Engine.Default.FontFamilies();

    /// <summary>Drop every registered font.</summary>
    public static void Reset() => Engine.Default.ResetFonts();
}
