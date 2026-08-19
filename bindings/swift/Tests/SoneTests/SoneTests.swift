import Foundation
import XCTest
@testable import Sone

final class SoneTests: XCTestCase {

    static let root = Native.checkoutRoot ?? FileManager.default.currentDirectoryPath
    static let font = "\(root)/fixtures/font/GeistMono-Regular.ttf"
    static let family = "Geist Mono"

    var engine: Engine!

    override func setUpWithError() throws {
        engine = Engine(Self.root)
        try engine.registerFontFile(Self.family, Self.font)
    }

    override func tearDown() {
        engine.close()
        engine = nil
    }

    private func props(_ node: Node) -> [String: IRValue] {
        Dictionary(uniqueKeysWithValues: node.props.entries.map { ($0.key, $0.value) })
    }

    private func json(_ node: Node) -> String { node.toJSON() }

    // MARK: - the builder, which touches no native code

    func testResultBuilderCollectsChildren() {
        let root = Column {
            Column().flex(1).cornerRadius(20).cornerSmoothing(0.7).bg("white")
            Row {
                Column().bg("lightgreen").size(50).borderRadius(14)
                Column().bg("salmon").height(50).borderRadius(14).flex(1)
            }.gap(10)
        }
        .gap(20).padding(20).size(420, 300).bg("khaki").cornerRadius(28)

        XCTAssertEqual(root.children.count, 2)
        XCTAssertEqual(root.children[1].children.count, 2)
        XCTAssertTrue(json(root).contains("\"gap\":20"))
    }

    func testIfAndForWorkInsideTheBuilder() {
        let labels = ["a", "b", "c"]
        let showEmpty = false
        let table = Table {
            for label in labels {
                TableRow { TableCell { Text(label) } }
            }
            if showEmpty {
                TableRow { TableCell {} }
            }
        }
        XCTAssertEqual(table.children.count, 3)
    }

    func testDimLiteralsCoverTheUnion() {
        let node = Column().width(100).minWidth("50%").maxWidth(.auto)
        let out = json(node)
        XCTAssertTrue(out.contains("\"width\":100"), out)
        XCTAssertTrue(out.contains("\"minWidth\":\"50%\""), out)
        XCTAssertTrue(out.contains("\"maxWidth\":\"auto\""), out)
    }

    func testSizeWithOneArgumentIsASquare() {
        let out = json(Column().size(50))
        XCTAssertTrue(out.contains("\"width\":50"), out)
        XCTAssertTrue(out.contains("\"height\":50"), out)
    }

    func testBoxShorthandFollowsCss() {
        let out = json(Column().padding(10, 20))
        XCTAssertTrue(out.contains("\"paddingTop\":10"), out)
        XCTAssertTrue(out.contains("\"paddingRight\":20"), out)
        XCTAssertTrue(out.contains("\"paddingLeft\":20"), out)
        XCTAssertFalse(out.contains("\"padding\":"), out)
    }

    func testLabelledSidesFillTheRestTheCssWay() {
        let out = json(Column().padding(top: 8, left: 4))
        XCTAssertTrue(out.contains("\"paddingTop\":8"), out)
        XCTAssertTrue(out.contains("\"paddingRight\":8"), out)
        XCTAssertTrue(out.contains("\"paddingLeft\":4"), out)
    }

    func testOneValueUsesTheShorthandProperty() {
        XCTAssertTrue(json(Column().margin(12)).contains("\"margin\":12"))
    }

    func testKeywordsUseLeadingDotSyntax() {
        let out = json(Row().justifyContent(.spaceBetween).alignItems(.center))
        XCTAssertTrue(out.contains("\"justifyContent\":\"space-between\""), out)
        XCTAssertTrue(out.contains("\"alignItems\":\"center\""), out)
    }

    func testBackgroundLayersAccumulateAndTakeAPhoto() {
        let out = json(Column().bg("red").bg(.photo(Photo("wall.png"))))
        XCTAssertTrue(out.contains("[\"red\",{\"type\":\"photo\""), out)
    }

    func testFiltersKeepTheOrderTheyWereAddedIn() {
        XCTAssertTrue(json(Column().blur(4).grayscale(0.5))
            .contains("[\"blur(4px)\",\"grayscale(0.5)\"]"))
    }

    func testTextSizeIsTheFontSizeNotTheBoxSize() {
        let out = json(Text("Hello").size(28))
        XCTAssertTrue(out.contains("\"size\":28"), out)
        XCTAssertFalse(out.contains("\"width\""), out)
    }

    func testTextTakesContentAndSpans() {
        let node = Text {
            "Hello "
            Span("world").weight(.bold).color("salmon")
        }
        let out = json(node)
        XCTAssertTrue(out.contains("\"inline\":[\"Hello \",{\"type\":\"span\""), out)
        XCTAssertTrue(out.contains("\"weight\":\"bold\""), out)
    }

    func testADecorationColourCanBeExplicitlyNull() {
        XCTAssertTrue(json(Text("x").underline().underlineColor())
            .contains("\"underlineColor\":null"))
    }

    func testGridTracksAcceptFrAndAuto() {
        let out = json(Grid().columns(.fr(1), .auto, 120))
        XCTAssertTrue(out.contains("[\"1fr\",\"auto\",120]"), out)
    }

    func testDocumentCarriesTheSchemaVersion() {
        let out = render(Column()).toJSON()
        XCTAssertTrue(out.hasPrefix("{\"sone\":1"), out)
        XCTAssertFalse(out.contains("\"config\""), out)
    }

    func testPaginationTokensArePassedThroughUntouched() {
        let out = render(Column(), pageHeight: 800, header: Text("Page {pageNumber}")).toJSON()
        XCTAssertTrue(out.contains("{pageNumber}"), out)
    }

    func testNonAsciiTextSurvivesUnescaped() {
        XCTAssertTrue(render(Text("អក្សរ")).toJSON().contains("អក្សរ"))
    }

    // MARK: - everything that crosses the C ABI

    func testRendersAPng() throws {
        let png = try render(Column().size(16).bg("red"), engine: engine).png()
        XCTAssertEqual([UInt8](png.prefix(4)), [0x89, 0x50, 0x4E, 0x47])
    }

    func testDensityScalesTheRaster() throws {
        let node = { Column().size(10).bg("red") }
        // Raw is 4 bytes per pixel, so the byte count is the pixel count.
        XCTAssertEqual(try render(node(), engine: engine).raw().count, 10 * 10 * 4)
        XCTAssertEqual(try render(node(), engine: engine).raw(density: 2).count, 20 * 20 * 4)
    }

    func testRendersEveryFormat() throws {
        let rendering = render(Column().size(16).bg("teal"), engine: engine)
        XCTAssertFalse(try rendering.jpeg(quality: 0.8).isEmpty)
        XCTAssertFalse(try rendering.webp().isEmpty)
        XCTAssertEqual(String(decoding: try rendering.pdf().prefix(4), as: UTF8.self), "%PDF")
        XCTAssertTrue(String(decoding: try rendering.svg(), as: UTF8.self).contains("<svg"))
    }

    func testOnePagePerDeclaredBreak() throws {
        let root = Column {
            Column().height(60).bg("red")
            Column().height(60).bg("green").pageBreak(.before)
            Column().height(60).bg("blue").pageBreak(.before)
        }
        let pages = try render(root, engine: engine, width: 40, pageHeight: 200).pages()
        XCTAssertEqual(pages.count, 3)
    }

    func testTheFontRegistryRoundTrips() throws {
        let fresh = Engine(Self.root)
        defer { fresh.close() }
        XCTAssertFalse(try fresh.hasFont(Self.family))
        try fresh.registerFontFile(Self.family, Self.font)
        XCTAssertTrue(try fresh.hasFont(Self.family))
        XCTAssertTrue(try fresh.fontFamilies().contains(Self.family))
        try fresh.resetFonts()
        XCTAssertFalse(try fresh.hasFont(Self.family))
        try fresh.registerFont(Self.family, Data(contentsOf: URL(fileURLWithPath: Self.font)))
        XCTAssertTrue(try fresh.hasFont(Self.family))
    }

    func testRegisteredImagesResolveAsAssets() throws {
        let png = try render(Column().size(8).bg("red"), engine: engine).png()
        try engine.registerImage("logo", png)
        XCTAssertFalse(try render(Photo("asset:logo").size(8), engine: engine).png().isEmpty)
    }

    func testLayoutComesBackAsJson() throws {
        let layout = try render(Column { Column().size(20).tag("inner") }.padding(5),
                                engine: engine).layoutJSON()
        XCTAssertTrue(layout.contains("\"width\":30.0"), layout)
        XCTAssertTrue(layout.contains("\"inner\""), layout)
    }

    func testMetadataHonoursGranularity() throws {
        let rendering = render(Text("hello world").font(Self.family).size(12), engine: engine)
        XCTAssertTrue(try rendering.metadataJSON().hasPrefix("{"))
        XCTAssertTrue(try rendering.metadataJSON(.word).hasPrefix("{"))
    }

    func testABadDocumentIsAnIrError() {
        XCTAssertThrowsError(try engine.render(#"{"sone":99,"root":{"type":"column"}}"#)) { error in
            guard let error = error as? SoneError else { return XCTFail("wrong error type") }
            XCTAssertEqual(error.kind, .ir)
            XCTAssertTrue(error.message.contains("unsupported IR version"), error.message)
        }
    }

    func testAMissingFontFileIsAnAssetError() {
        XCTAssertThrowsError(try engine.registerFontFile("Nope", "does/not/exist.ttf")) { error in
            XCTAssertEqual((error as? SoneError)?.kind, .asset)
        }
    }

    func testUsingAClosedEngineThrowsRatherThanCrashing() {
        let closed = Engine(Self.root)
        closed.close()
        closed.close()
        XCTAssertThrowsError(try closed.hasFont(Self.family))
    }

    func testSaveInfersTheFormatFromTheExtension() throws {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("sone-swift-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }

        let path = directory.appendingPathComponent("card.pdf").path
        try render(Column().size(16).bg("red"), engine: engine).save(path)
        let bytes = try Data(contentsOf: URL(fileURLWithPath: path))
        XCTAssertEqual(String(decoding: bytes.prefix(4), as: UTF8.self), "%PDF")
    }

    /// The gate every binding owes: the same document must come out of this
    /// binding byte for byte the way it comes out of `sone-cli`.
    ///
    /// macOS only — `Process` does not exist on iOS, so there is no way to run
    /// the CLI from inside the test bundle. Everything above this line runs on
    /// iPhone and iPad simulators too.
    func testMatchesTheCliByteForByte() throws {
        #if !os(macOS)
        throw XCTSkip("no Process on this platform")
        #else
        let root = Column {
            Text {
                "Hello "
                Span("world").weight(.bold).color("#c0392b")
            }
            .font(Self.family).size(24).lineHeight(1.4)

            Row {
                Column().bg("lightgreen").size(50).borderRadius(14)
                Column().bg("salmon").height(50).borderRadius(14).flex(1)
            }.gap(10)
        }
        .gap(20).padding(20).size(420, 200).bg("khaki").cornerRadius(28)

        // An absolute src, because the CLI resolves a document's assets against
        // the document's own directory and the engine resolves them against its
        // base directory — the two only agree when the path is absolute.
        let rendering = render(root, engine: engine, density: 2,
                               fonts: [FontSource(Self.family, Self.font)])

        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent("sone-parity-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }

        let document = directory.appendingPathComponent("doc.json")
        let fromCli = directory.appendingPathComponent("cli.png")
        try rendering.toJSON().write(to: document, atomically: true, encoding: .utf8)

        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = ["cargo", "run", "-q", "-p", "sone-cli", "--", "render",
                             document.path, "--density", "2", "-o", fromCli.path]
        process.currentDirectoryURL = URL(fileURLWithPath: Self.root)
        try process.run()
        process.waitUntilExit()
        XCTAssertEqual(process.terminationStatus, 0)

        XCTAssertEqual(try Data(contentsOf: fromCli), try rendering.png())
        #endif
    }
}
