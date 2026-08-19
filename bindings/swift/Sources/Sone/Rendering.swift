import Foundation

/// A node plus its render configuration, with one method per output format.
public struct Rendering {
    let root: Node
    var engine: Engine?
    var width: Double?
    var height: Double?
    var background: String?
    var density: Double?
    var pageHeight: Double?
    var margin: Margin?
    var lastPageHeight: LastPageHeight?

    /// Drawn at the top of every page. Use the literal tokens `{pageNumber}` and
    /// `{totalPages}` — the engine substitutes them.
    var header: Node?

    /// Drawn at the bottom of every page.
    var footer: Node?

    /// Fonts the document carries with it, so another sone engine — the CLI,
    /// say — renders it identically.
    var fonts: [FontSource] = []

    var resolvedEngine: Engine { engine ?? Engine.shared }

    // MARK: - the document

    /// The IR document as JSON.
    public func toJSON() -> String {
        var config: [(String, IRValue)] = []
        if let width { config.append(("width", .number(width))) }
        if let height { config.append(("height", .number(height))) }
        if let background { config.append(("background", .string(background))) }
        if let density { config.append(("density", .number(density))) }
        if let pageHeight { config.append(("pageHeight", .number(pageHeight))) }
        if let margin { config.append(("margin", margin.ir)) }
        if let lastPageHeight { config.append(("lastPageHeight", .string(lastPageHeight.rawValue))) }
        if let header { config.append(("header", .node(header))) }
        if let footer { config.append(("footer", .node(footer))) }

        var out = "{\"sone\":1"
        if !fonts.isEmpty {
            out += ",\"fonts\":"
            Json.write(.array(fonts.map { .object([("name", .string($0.name)), ("src", .string($0.src))]) }),
                       into: &out)
        }
        if !config.isEmpty {
            out += ",\"config\":"
            Json.write(.object(config), into: &out)
        }
        out += ",\"root\":"
        root.write(into: &out)
        out += "}"
        return out
    }

    // MARK: - outputs

    public func png(density: Double? = nil) throws -> Data {
        try resolvedEngine.render(toJSON(), format: .png, density: density)
    }

    public func jpeg(quality: Double = 1.0, density: Double? = nil) throws -> Data {
        try resolvedEngine.render(toJSON(), format: .jpeg, density: density, quality: quality)
    }

    public func webp(quality: Double = 1.0, density: Double? = nil) throws -> Data {
        try resolvedEngine.render(toJSON(), format: .webp, density: density, quality: quality)
    }

    /// Raw RGBA pixels, row-major, unpremultiplied.
    public func raw(density: Double? = nil) throws -> Data {
        try resolvedEngine.render(toJSON(), format: .raw, density: density)
    }

    /// A PDF. With `pageHeight` set, one page per break and selectable text.
    public func pdf() throws -> Data {
        try resolvedEngine.render(toJSON(), format: .pdf)
    }

    public func svg() throws -> Data {
        try resolvedEngine.render(toJSON(), format: .svg)
    }

    /// One raster image per page. Requires `pageHeight`.
    public func pages(format: OutputFormat = .png, density: Double? = nil, quality: Double = 1.0) throws -> [Data] {
        try resolvedEngine.renderPages(toJSON(), format: format, density: density, quality: quality)
    }

    /// Render and write to `path`, inferring the format from its extension.
    @discardableResult
    public func save(_ path: String, density: Double? = nil, quality: Double = 1.0) throws -> String {
        let bytes: Data = switch try Self.format(for: path) {
        case .png: try png(density: density)
        case .jpeg: try jpeg(quality: quality, density: density)
        case .webp: try webp(quality: quality, density: density)
        case .raw: try raw(density: density)
        case .pdf: try pdf()
        case .svg: try svg()
        }
        try bytes.write(to: URL(fileURLWithPath: path))
        return path
    }

    /// Write `name-1.png`, `name-2.png`, … next to `path`.
    @discardableResult
    public func savePages(_ path: String, density: Double? = nil, quality: Double = 1.0) throws -> [String] {
        let url = URL(fileURLWithPath: path)
        let ext = url.pathExtension
        let stem = url.deletingPathExtension().path
        let format = (try? Self.format(for: path)) ?? .png

        return try pages(format: format, density: density, quality: quality)
            .enumerated()
            .map { index, bytes in
                let name = ext.isEmpty ? "\(stem)-\(index + 1).png" : "\(stem)-\(index + 1).\(ext)"
                try bytes.write(to: URL(fileURLWithPath: name))
                return name
            }
    }

    // MARK: - introspection

    /// The computed layout tree, as JSON.
    public func layoutJSON() throws -> String {
        try resolvedEngine.dumpLayout(toJSON())
    }

    /// Dataset-style boxes at node, line or word granularity, as JSON.
    public func metadataJSON(_ granularity: Granularity = .node) throws -> String {
        try resolvedEngine.dumpMetadata(toJSON(), granularity: granularity)
    }

    private static func format(for path: String) throws -> OutputFormat {
        switch URL(fileURLWithPath: path).pathExtension.lowercased() {
        case "png": return .png
        case "jpg", "jpeg": return .jpeg
        case "webp": return .webp
        case "pdf": return .pdf
        case "svg": return .svg
        case "raw", "rgba": return .raw
        default:
            throw SoneError(kind: .invalidArgument,
                            message: "cannot infer an output format from \"\(path)\"")
        }
    }
}

/// Wrap a node with render configuration.
///
///     try render(root, density: 2).save("card.png")
public func render(
    _ root: Node,
    engine: Engine? = nil,
    width: Double? = nil,
    height: Double? = nil,
    background: String? = nil,
    density: Double? = nil,
    pageHeight: Double? = nil,
    margin: Margin? = nil,
    lastPageHeight: LastPageHeight? = nil,
    header: Node? = nil,
    footer: Node? = nil,
    fonts: [FontSource] = []
) -> Rendering {
    Rendering(root: root, engine: engine, width: width, height: height,
              background: background, density: density, pageHeight: pageHeight,
              margin: margin, lastPageHeight: lastPageHeight,
              header: header, footer: footer, fonts: fonts)
}
