package dev.sone;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * Where the native library is.
 *
 * <p>Shared by both backends, because "find libsone" is the same question on
 * every platform even though the two answer it with different call mechanisms.
 */
public final class LibraryPath {

    /** A full path to the library, or a directory holding it. */
    public static final String PATH_VARIABLE = "SONE_NATIVE_LIBRARY";

    private LibraryPath() {
    }

    /** Whether this is running on Android rather than a desktop JVM. */
    public static boolean isAndroid() {
        try {
            Class.forName("android.os.Build");
            return true;
        } catch (ClassNotFoundException e) {
            return false;
        }
    }

    public static String fileName() {
        String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        if (os.contains("win")) {
            return "sone.dll";
        }
        return os.contains("mac") ? "libsone.dylib" : "libsone.so";
    }

    /**
     * An explicit hint first, then a {@code cargo build} in a checkout, then the
     * bare name for the platform loader to resolve.
     *
     * <p>On Android there is nothing to search: the library is unpacked from the
     * APK into the app's {@code nativeLibraryDir}, which the loader already
     * knows about, so the bare name is the only answer.
     */
    public static String locate() {
        if (isAndroid()) {
            return "sone";
        }

        String name = fileName();
        List<Path> candidates = new ArrayList<>();

        String hint = System.getenv(PATH_VARIABLE);
        if (hint != null && !hint.isEmpty()) {
            Path path = Path.of(hint);
            candidates.add(Files.isDirectory(path) ? path.resolve(name) : path);
        }
        Path root = checkoutRoot();
        if (root != null) {
            candidates.add(root.resolve("target/release/" + name));
            candidates.add(root.resolve("target/debug/" + name));
        }
        for (Path candidate : candidates) {
            if (Files.isRegularFile(candidate)) {
                return candidate.toString();
            }
        }
        return name;
    }

    /** The repository root, when this artifact is used from a checkout. */
    public static Path checkoutRoot() {
        if (isAndroid()) {
            return null;
        }
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            if (Files.isRegularFile(directory.resolve("Cargo.toml"))
                    && Files.isDirectory(directory.resolve("crates"))) {
                return directory;
            }
            directory = directory.getParent();
        }
        return null;
    }
}
