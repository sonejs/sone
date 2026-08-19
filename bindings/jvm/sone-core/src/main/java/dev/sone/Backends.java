package dev.sone;

import java.lang.reflect.Constructor;
import java.util.List;

/**
 * Finds the backend for the platform this is running on.
 *
 * <p>Panama first, JNA second, and the order matters: on a desktop JVM both may
 * be on the classpath, and Panama is the one without a third-party dependency.
 * On Android the Panama module is simply not present.
 */
public final class Backends {

    private static final List<String> CANDIDATES = List.of(
            "dev.sone.panama.PanamaBackend",
            "dev.sone.jna.JnaBackend");

    private Backends() {
    }

    /** The name of the backend that would be used, for diagnostics. */
    public static String describe() {
        for (String name : CANDIDATES) {
            if (isUsable(name)) {
                return name;
            }
        }
        return "none";
    }

    /** Pin a backend by class name, for tests and for diagnosing a bad pick. */
    public static final String PROPERTY = "sone.backend";

    /** Build a named backend, bypassing discovery. */
    public static Backend create(String name, String baseDir) {
        return construct(name, baseDir);
    }

    static Backend create(String baseDir) {
        String pinned = System.getProperty(PROPERTY);
        if (pinned != null && !pinned.isEmpty()) {
            return construct(pinned, baseDir);
        }
        Throwable last = null;
        for (String name : CANDIDATES) {
            try {
                Constructor<?> constructor = Class.forName(name).getConstructor(String.class);
                return (Backend) constructor.newInstance(baseDir);
            } catch (ClassNotFoundException | NoClassDefFoundError e) {
                last = e; // not on this classpath, or its own dependency is missing
            } catch (ReflectiveOperationException e) {
                Throwable cause = e.getCause() != null ? e.getCause() : e;
                if (cause instanceof RuntimeException runtime) {
                    throw runtime; // a real failure — the library is missing, say so
                }
                last = cause;
            }
        }
        throw new SoneException(
                "no sone backend on the classpath. Add sone-panama (desktop, Java 22+) "
                        + "or sone-jna (Android, and anywhere Panama is unavailable)."
                        + (last == null ? "" : " Last attempt: " + last));
    }

    private static Backend construct(String name, String baseDir) {
        try {
            Constructor<?> constructor = Class.forName(name).getConstructor(String.class);
            return (Backend) constructor.newInstance(baseDir);
        } catch (ReflectiveOperationException e) {
            Throwable cause = e.getCause() != null ? e.getCause() : e;
            if (cause instanceof RuntimeException runtime) {
                throw runtime;
            }
            throw new SoneException("could not create " + name + ": " + cause);
        }
    }

    private static boolean isUsable(String name) {
        try {
            Class.forName(name);
            return true;
        } catch (ClassNotFoundException | NoClassDefFoundError e) {
            return false;
        }
    }
}
