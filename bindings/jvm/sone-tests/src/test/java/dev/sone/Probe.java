package dev.sone;

import java.nio.file.Path;

/**
 * Walks the C ABI one call at a time, printing before and after each.
 *
 * <p>Not a test: a JVM crash loses whatever surefire had buffered, so this
 * writes to stderr and flushes, and takes the backend as an argument so each one
 * runs in a JVM of its own.
 *
 * <pre>
 * java -cp … dev.sone.Probe dev.sone.panama.PanamaBackend
 * </pre>
 */
public final class Probe {

    public static void main(String[] args) {
        String name = args.length > 0 ? args[0] : "dev.sone.panama.PanamaBackend";
        Path root = LibraryPath.checkoutRoot();
        String family = "Geist Mono";
        String font = root.resolve("fixtures/font/GeistMono-Regular.ttf").toString();

        say(name, "library = " + LibraryPath.locate());

        try (Backend backend = Backends.create(name, root.toString())) {
            say(name, "created");

            backend.registerFontFile(family, font);
            say(name, "registerFontFile ok");

            say(name, "fontFamilies = " + backend.fontFamilies());

            String shape = Sone.render(new Column().size(16, 16).bg("red")).toJson();
            say(name, "render(shape) = " + backend.render(shape, OutputFormat.PNG, null, 1.0, false).length);

            String text = Sone.render(new Text("hello world").font(family).size(12)).toJson();
            say(name, "render(text) = " + backend.render(text, OutputFormat.PNG, null, 1.0, false).length);

            say(name, "dumpLayout(shape) = " + backend.dumpLayout(shape).length());
            say(name, "dumpLayout(text) = " + backend.dumpLayout(text).length());

            say(name, "dumpMetadata(shape, NODE) = " + backend.dumpMetadata(shape, Granularity.NODE).length());
            say(name, "dumpMetadata(text, NODE) = " + backend.dumpMetadata(text, Granularity.NODE).length());
            say(name, "dumpMetadata(text, WORD) = " + backend.dumpMetadata(text, Granularity.WORD).length());
        } catch (Throwable e) {
            say(name, "threw " + e);
            e.printStackTrace();
            System.exit(1);
        }
        say(name, "closed cleanly");
    }

    private static void say(String backend, String message) {
        System.err.println("PROBE [" + backend + "] " + message);
        System.err.flush();
    }
}
