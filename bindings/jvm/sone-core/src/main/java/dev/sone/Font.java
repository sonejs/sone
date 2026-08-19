package dev.sone;

import java.util.List;

/**
 * Font registration on the process-wide engine, for programs that do not want
 * to own one.
 *
 * <p>Skia carries no system fonts, so at least one family must be registered
 * before any text renders.
 */
public final class Font {

    private Font() {
    }

    public static void load(String name, String path) {
        Engine.getDefault().registerFontFile(name, path);
    }

    public static void load(String name, byte[] data) {
        Engine.getDefault().registerFont(name, data);
    }

    public static boolean has(String name) {
        return Engine.getDefault().hasFont(name);
    }

    public static List<String> families() {
        return Engine.getDefault().fontFamilies();
    }

    public static void reset() {
        Engine.getDefault().resetFonts();
    }
}
