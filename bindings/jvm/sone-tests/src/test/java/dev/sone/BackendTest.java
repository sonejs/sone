package dev.sone;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

/**
 * The two backends, held to each other.
 *
 * <p>Android runs the JNA one and cannot run the Panama one, so this is where
 * the Android call path gets exercised without an Android device: same C ABI,
 * same class, and the bytes have to match.
 */
class BackendTest {

    /**
     * Windows cannot host both loaders at once.
     *
     * <p>Each backend passes the whole suite on Windows on its own — CI pins one
     * per JVM and both are green. What crashes is loading the same DLL through
     * Panama and JNA in a single process, which no consumer does: a desktop app
     * ships sone-panama and an Android app ships sone-jna, never both. Rather
     * than pretend the comparison covers Windows, it says so.
     */
    @BeforeAll
    static void bothLoadersInOneProcess() {
        Assumptions.assumeFalse(
                System.getProperty("os.name", "").toLowerCase(java.util.Locale.ROOT).contains("win"),
                "Panama and JNA cannot both map the same DLL in one JVM on Windows");
    }

    static final Path ROOT = LibraryPath.checkoutRoot();
    static final String FAMILY = "Geist Mono";
    static final Path FONT = ROOT.resolve("fixtures/font/GeistMono-Regular.ttf");

    static final String PANAMA = "dev.sone.panama.PanamaBackend";
    static final String JNA = "dev.sone.jna.JnaBackend";

    private static Backend backend(String name) {
        return Backends.create(name, ROOT.toString());
    }

    private static Column card() {
        return new Column(
                new Text("Hello ", new Span("world").weight("bold").color("#c0392b"))
                        .font(FAMILY).size(24).lineHeight(1.4),
                new Row(
                        new Column().bg("lightgreen").size(50).borderRadius(14),
                        new Column().bg("salmon").height(50).borderRadius(14).flex(1))
                        .gap(10))
                .gap(20).padding(20).size(420, 200).bg("khaki").cornerRadius(28);
    }

    @Test
    void bothBackendsAreOnTheClasspath() {
        assertEquals(PANAMA, Backends.describe(), "Panama should win when both are present");
    }

    @ParameterizedTest
    @ValueSource(strings = {PANAMA, JNA})
    void rendersAPng(String name) {
        try (Backend backend = backend(name)) {
            backend.registerFontFile(FAMILY, FONT.toString());
            byte[] png = backend.render(Sone.render(card()).toJson(), OutputFormat.PNG, 2.0, 1.0, false);
            assertArrayEquals(new byte[] {(byte) 0x89, 'P', 'N', 'G'},
                    java.util.Arrays.copyOf(png, 4));
        }
    }

    @ParameterizedTest
    @ValueSource(strings = {PANAMA, JNA})
    void theFontRegistryRoundTrips(String name) {
        try (Backend backend = backend(name)) {
            assertFalse(backend.hasFont(FAMILY));
            backend.registerFontFile(FAMILY, FONT.toString());
            assertTrue(backend.hasFont(FAMILY));
            assertTrue(backend.fontFamilies().contains(FAMILY));
            backend.resetFonts();
            assertFalse(backend.hasFont(FAMILY));
        }
    }

    @ParameterizedTest
    @ValueSource(strings = {PANAMA, JNA})
    void onePagePerDeclaredBreak(String name) {
        try (Backend backend = backend(name)) {
            Column root = new Column(
                    new Column().height(60).bg("red"),
                    new Column().height(60).bg("green").pageBreak(PageBreakMode.BEFORE),
                    new Column().height(60).bg("blue").pageBreak(PageBreakMode.BEFORE));
            String document = Sone.render(root).width(40).pageHeight(200).toJson();
            List<byte[]> pages = backend.renderPages(document, OutputFormat.PNG, null, 1.0, false);
            assertEquals(3, pages.size());
            for (byte[] page : pages) {
                assertTrue(page.length > 0);
            }
        }
    }

    @ParameterizedTest
    @ValueSource(strings = {PANAMA, JNA})
    void errorsMapToTheSameTypes(String name) {
        try (Backend backend = backend(name)) {
            IrException ir = assertThrows(IrException.class, () -> backend.render(
                    "{\"sone\":99,\"root\":{\"type\":\"column\"}}", OutputFormat.PNG, null, 1.0, false));
            assertTrue(ir.getMessage().contains("unsupported IR version"), ir.getMessage());
            assertThrows(AssetException.class,
                    () -> backend.registerFontFile("Nope", "does/not/exist.ttf"));
        }
    }

    /**
     * The gate for the Android path: two different call mechanisms over one C
     * ABI have to produce identical bytes, or Android is quietly rendering
     * something else.
     */
    @Test
    void theTwoBackendsProduceIdenticalBytes() {
        String document = Sone.render(card()).density(2).toJson();
        byte[] fromPanama;
        byte[] fromJna;
        String layoutPanama;
        String layoutJna;

        try (Backend backend = backend(PANAMA)) {
            backend.registerFontFile(FAMILY, FONT.toString());
            fromPanama = backend.render(document, OutputFormat.PNG, 2.0, 1.0, false);
            layoutPanama = backend.dumpLayout(document);
        }
        try (Backend backend = backend(JNA)) {
            backend.registerFontFile(FAMILY, FONT.toString());
            fromJna = backend.render(document, OutputFormat.PNG, 2.0, 1.0, false);
            layoutJna = backend.dumpLayout(document);
        }

        assertTrue(fromPanama.length > 1000, "expected a real PNG");
        assertArrayEquals(fromPanama, fromJna);
        assertEquals(layoutPanama, layoutJna);
    }

    @Test
    void bothReportTheSameVersion() {
        try (Backend panama = backend(PANAMA); Backend jna = backend(JNA)) {
            assertEquals(panama.version(), jna.version());
        }
    }

    @ParameterizedTest
    @ValueSource(strings = {PANAMA, JNA})
    void utf8SurvivesTheBoundary(String name) {
        try (Backend backend = backend(name)) {
            backend.registerFontFile(FAMILY, FONT.toString());
            String document = Sone.render(new Text("អក្សរ ← Khmer").font(FAMILY).size(14)).toJson();
            String metadata = backend.dumpMetadata(document, Granularity.NODE);
            assertTrue(metadata.getBytes(StandardCharsets.UTF_8).length > 0);
        }
    }
}
