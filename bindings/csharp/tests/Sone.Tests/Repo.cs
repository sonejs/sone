namespace Sone.Tests;

/// <summary>
/// Locates the checkout, and points the binding at the library <c>cargo build</c>
/// produced. NuGet consumers get the native library from
/// <c>runtimes/{rid}/native</c>; running from source there is nothing there to
/// find, so the tests set the hint the loader reads.
/// </summary>
internal static class Repo
{
    static Repo() => UseCargoBuild();

    private static void UseCargoBuild()
    {
        if (Environment.GetEnvironmentVariable(NativeLoaderHint) is { Length: > 0 })
        {
            return;
        }
        foreach (var profile in new[] { "release", "debug" })
        {
            var candidate = Path.Combine(Root, "target", profile, LibraryName);
            if (File.Exists(candidate))
            {
                Environment.SetEnvironmentVariable(NativeLoaderHint, candidate);
                return;
            }
        }
    }

    private const string NativeLoaderHint = "SONE_NATIVE_LIBRARY";

    private static string LibraryName =>
        OperatingSystem.IsWindows() ? "sone.dll" : OperatingSystem.IsMacOS() ? "libsone.dylib" : "libsone.so";

    /// <summary>The repository root, found by walking up to the workspace Cargo.toml.</summary>
    internal static string Root => RootValue;

    private static readonly string RootValue = FindRoot();

    internal static string Fixture(string relative) => Path.Combine(Root, relative);

    /// <summary>Whether the native library is present, so tests can skip rather than fail.</summary>
    internal static bool HasNative =>
        Environment.GetEnvironmentVariable(NativeLoaderHint) is { Length: > 0 };

    private static string FindRoot()
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
