namespace Sone.Sample;

/// <summary>
/// Finds the checkout, and points the binding at the library <c>cargo build</c>
/// produced. A NuGet consumer needs none of this — the native library ships in
/// <c>runtimes/{rid}/native</c> and the default resolver finds it there.
/// </summary>
internal static class Checkout
{
    internal static string Root { get; } = Find();

    internal static void UseLocalNativeLibrary()
    {
        const string variable = "SONE_NATIVE_LIBRARY";
        if (Environment.GetEnvironmentVariable(variable) is { Length: > 0 })
        {
            return;
        }

        var name = OperatingSystem.IsWindows() ? "sone.dll"
            : OperatingSystem.IsMacOS() ? "libsone.dylib"
            : "libsone.so";

        foreach (var profile in new[] { "release", "debug" })
        {
            var candidate = Path.Combine(Root, "target", profile, name);
            if (File.Exists(candidate))
            {
                Environment.SetEnvironmentVariable(variable, candidate);
                return;
            }
        }
        throw new FileNotFoundException($"no {name} in target/release or target/debug — run `cargo build -p sone-ffi`");
    }

    private static string Find()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);
        while (directory is not null)
        {
            if (File.Exists(Path.Combine(directory.FullName, "Cargo.toml"))
                && Directory.Exists(Path.Combine(directory.FullName, "crates")))
            {
                return directory.FullName;
            }
            directory = directory.Parent;
        }
        throw new InvalidOperationException("could not find the repository root from " + AppContext.BaseDirectory);
    }
}

/// <summary>Just enough PDF reading to report a page count.</summary>
internal static class Pdf
{
    internal static int CountPages(string path)
    {
        var text = System.Text.Encoding.Latin1.GetString(File.ReadAllBytes(path));
        var count = 0;
        var index = 0;
        while ((index = text.IndexOf("/Type /Page", index, StringComparison.Ordinal)) >= 0)
        {
            // "/Type /Pages" is the tree root, not a page.
            if (index + 11 >= text.Length || text[index + 11] != 's')
            {
                count++;
            }
            index += 11;
        }
        return count;
    }
}
