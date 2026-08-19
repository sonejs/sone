package dev.sone;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class SoneTest {

    static final Path ROOT = LibraryPath.checkoutRoot();
    static final String FAMILY = "Geist Mono";
    static final Path FONT = ROOT.resolve("fixtures/font/GeistMono-Regular.ttf");

    Engine engine;

    @BeforeEach
    void setUp() {
        engine = new Engine(ROOT.toString());
        engine.registerFontFile(FAMILY, FONT.toString());
    }

    @AfterEach
    void tearDown() {
        engine.close();
    }

    // ── the builder, which touches no native code ───────────────────────────

    @Test
    void chainingKeepsTheConcreteType() {
        // If SELF stopped inferring, this would only compile as LayoutProps.
        Column node = new Column().gap(20).padding(20).bg("khaki").rounded(8);
        assertTrue(node.toJson().contains("\"gap\":20"));
    }

    @Test
    void constructorsTakeChildren() {
        Column root = new Column(
                new Column().flex(1).cornerRadius(20).bg("white"),
                new Row(new Column().bg("salmon").size(50)).gap(10));
        assertEquals(2, root.children().size());
    }

    @Test
    void nullChildrenAreDropped() {
        boolean show = false;
        assertEquals(1, new Column(new Column(), show ? new Row() : null).children().size());
    }

    @Test
    void dimCoversAutoAndPercentages() {
        String json = new Column().width(100).minWidth(Dim.percent(50)).maxWidth(Dim.AUTO).toJson();
        assertTrue(json.contains("\"width\":100"), json);
        assertTrue(json.contains("\"minWidth\":\"50%\""), json);
        assertTrue(json.contains("\"maxWidth\":\"auto\""), json);
    }

    @Test
    void sizeWithOneArgumentIsASquare() {
        String json = new Column().size(50).toJson();
        assertTrue(json.contains("\"width\":50"), json);
        assertTrue(json.contains("\"height\":50"), json);
    }

    @Test
    void arityOverloadsStandInForNamedArguments() {
        String json = new Column().padding(10, 20).toJson();
        assertTrue(json.contains("\"paddingTop\":10"), json);
        assertTrue(json.contains("\"paddingRight\":20"), json);
        assertTrue(json.contains("\"paddingLeft\":20"), json);
        assertFalse(json.contains("\"padding\":"), json);
    }

    @Test
    void oneValueUsesTheShorthandProperty() {
        assertTrue(new Column().margin(12).toJson().contains("\"margin\":12"));
    }

    @Test
    void keywordsAreEnums() {
        String json = new Row().justifyContent(JustifyContent.SPACE_BETWEEN)
                .alignItems(AlignItems.CENTER).toJson();
        assertTrue(json.contains("\"justifyContent\":\"space-between\""), json);
        assertTrue(json.contains("\"alignItems\":\"center\""), json);
    }

    @Test
    void backgroundLayersAccumulateAndTakeAPhoto() {
        String json = new Column().bg("red").bg(new Photo("wall.png")).toJson();
        assertTrue(json.contains("[\"red\",{\"type\":\"photo\""), json);
    }

    @Test
    void filtersKeepTheOrderTheyWereAddedIn() {
        assertTrue(new Column().blur(4).grayscale(0.5).toJson()
                .contains("[\"blur(4px)\",\"grayscale(0.5)\"]"));
    }

    @Test
    void textSizeIsTheFontSizeNotTheBoxSize() {
        // Two interfaces declare size(double), so the compiler forces Text to
        // override — which is where the rule lives.
        String json = new Text("Hello").size(28).toJson();
        assertTrue(json.contains("\"size\":28"), json);
        assertFalse(json.contains("\"width\""), json);
    }

    @Test
    void textTakesStringsAndSpans() {
        String json = new Text("Hello ", new Span("world").weight("bold")).toJson();
        assertTrue(json.contains("\"inline\":[\"Hello \",{\"type\":\"span\""), json);
        assertTrue(json.contains("\"weight\":\"bold\""), json);
    }

    @Test
    void aDecorationColourCanBeExplicitlyNull() {
        assertTrue(new Text("x").underline().underlineColor().toJson()
                .contains("\"underlineColor\":null"));
    }

    @Test
    void gridTracksAcceptFrAndAuto() {
        String json = new Grid().columns(Track.fr(1), Track.AUTO, Track.of(120)).toJson();
        assertTrue(json.contains("[\"1fr\",\"auto\",120]"), json);
    }

    @Test
    void listAndPathAreRenamedForTheJdk() {
        // java.util.List and java.nio.file.Path own the obvious names.
        assertEquals("list", new Bullets().type());
        assertEquals("path", new SvgPath("M0 0").type());
    }

    @Test
    void theDocumentCarriesTheSchemaVersion() {
        String json = Sone.render(new Column()).toJson();
        assertTrue(json.startsWith("{\"sone\":1"), json);
        assertFalse(json.contains("\"config\""), json);
    }

    @Test
    void paginationTokensArePassedThroughUntouched() {
        String json = Sone.render(new Column()).pageHeight(800)
                .header(new Text("Page {pageNumber}")).toJson();
        assertTrue(json.contains("{pageNumber}"), json);
    }

    @Test
    void nonAsciiTextSurvivesUnescaped() {
        assertTrue(Sone.render(new Text("អក្សរ")).toJson().contains("អក្សរ"));
    }

    // ── everything that crosses the C ABI ───────────────────────────────────

    @Test
    void rendersAPng() {
        byte[] png = Sone.render(new Column().size(16).bg("red")).engine(engine).png();
        assertArrayEquals(new byte[] {(byte) 0x89, 'P', 'N', 'G'}, java.util.Arrays.copyOf(png, 4));
    }

    @Test
    void densityScalesTheRaster() {
        // Raw is 4 bytes per pixel, so the byte count is the pixel count.
        assertEquals(10 * 10 * 4, Sone.render(new Column().size(10).bg("red")).engine(engine).raw().length);
        assertEquals(20 * 20 * 4, Sone.render(new Column().size(10).bg("red")).engine(engine).raw(2.0).length);
    }

    @Test
    void rendersEveryFormat() {
        Rendering rendering = Sone.render(new Column().size(16).bg("teal")).engine(engine);
        assertTrue(rendering.jpeg(0.8).length > 0);
        assertTrue(rendering.webp(1.0).length > 0);
        assertEquals("%PDF", new String(java.util.Arrays.copyOf(rendering.pdf(), 4), StandardCharsets.US_ASCII));
        assertTrue(new String(rendering.svg(), StandardCharsets.UTF_8).contains("<svg"));
    }

    @Test
    void onePagePerDeclaredBreak() {
        Column root = new Column(
                new Column().height(60).bg("red"),
                new Column().height(60).bg("green").pageBreak(PageBreakMode.BEFORE),
                new Column().height(60).bg("blue").pageBreak(PageBreakMode.BEFORE));
        List<byte[]> pages = Sone.render(root).engine(engine).width(40).pageHeight(200).pages();
        assertEquals(3, pages.size());
    }

    @Test
    void theFontRegistryRoundTrips() {
        try (Engine fresh = new Engine(ROOT.toString())) {
            assertFalse(fresh.hasFont(FAMILY));
            fresh.registerFontFile(FAMILY, FONT.toString());
            assertTrue(fresh.hasFont(FAMILY));
            assertTrue(fresh.fontFamilies().contains(FAMILY));
            fresh.resetFonts();
            assertFalse(fresh.hasFont(FAMILY));
        }
    }

    @Test
    void registeredImagesResolveAsAssets() throws IOException {
        byte[] png = Sone.render(new Column().size(8).bg("red")).engine(engine).png();
        engine.registerImage("logo", png);
        assertTrue(Sone.render(new Photo("asset:logo").size(8)).engine(engine).png().length > 0);
    }

    @Test
    void layoutComesBackAsJson() {
        String layout = Sone.render(new Column(new Column().size(20).tag("inner")).padding(5))
                .engine(engine).layoutJson();
        assertTrue(layout.contains("\"width\":30.0"), layout);
        assertTrue(layout.contains("\"inner\""), layout);
    }

    @Test
    void metadataHonoursGranularity() {
        Rendering rendering = Sone.render(new Text("hello world").font(FAMILY).size(12)).engine(engine);
        assertTrue(rendering.metadataJson().startsWith("{"));
        assertTrue(rendering.metadataJson(Granularity.WORD).startsWith("{"));
    }

    @Test
    void aBadDocumentIsAnIrError() {
        IrException error = assertThrows(IrException.class,
                () -> engine.render("{\"sone\":99,\"root\":{\"type\":\"column\"}}", OutputFormat.PNG));
        assertTrue(error.getMessage().contains("unsupported IR version"), error.getMessage());
    }

    @Test
    void aMissingFontFileIsAnAssetError() {
        assertThrows(AssetException.class, () -> engine.registerFontFile("Nope", "does/not/exist.ttf"));
    }

    @Test
    void usingAClosedEngineThrowsRatherThanCrashing() {
        Engine closed = new Engine(ROOT.toString());
        closed.close();
        closed.close();
        assertThrows(IllegalStateException.class, () -> closed.hasFont(FAMILY));
    }

    @Test
    void saveInfersTheFormatFromTheExtension() throws IOException {
        Path directory = Files.createTempDirectory("sone-jvm");
        Path target = Sone.render(new Column().size(16).bg("red")).engine(engine)
                .save(directory.resolve("card.pdf"));
        assertEquals("%PDF", new String(java.util.Arrays.copyOf(Files.readAllBytes(target), 4),
                StandardCharsets.US_ASCII));
    }

    /**
     * The gate every binding owes: the same document must come out of this
     * binding byte for byte the way it comes out of {@code sone-cli}.
     */
    @Test
    void matchesTheCliByteForByte() throws Exception {
        Column root = new Column(
                new Text("Hello ", new Span("world").weight("bold").color("#c0392b"))
                        .font(FAMILY).size(24).lineHeight(1.4),
                new Row(
                        new Column().bg("lightgreen").size(50).borderRadius(14),
                        new Column().bg("salmon").height(50).borderRadius(14).flex(1))
                        .gap(10))
                .gap(20).padding(20).size(420, 200).bg("khaki").cornerRadius(28);

        // An absolute src, because the CLI resolves a document's assets against
        // the document's own directory and the engine resolves them against its
        // base directory — the two only agree when the path is absolute.
        Rendering rendering = Sone.render(root).engine(engine).density(2)
                .font(FAMILY, FONT.toString());

        Path directory = Files.createTempDirectory("sone-parity");
        Path document = directory.resolve("doc.json");
        Path fromCli = directory.resolve("cli.png");
        Files.writeString(document, rendering.toJson());

        Process process = new ProcessBuilder("cargo", "run", "-q", "-p", "sone-cli", "--",
                "render", document.toString(), "--density", "2", "-o", fromCli.toString())
                .directory(ROOT.toFile())
                .redirectErrorStream(true)
                .start();
        String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
        assertEquals(0, process.waitFor(), output);

        assertArrayEquals(Files.readAllBytes(fromCli), rendering.png());
    }
}
