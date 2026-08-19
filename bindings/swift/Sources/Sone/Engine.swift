import CSone
import Foundation

/// The base for every sone failure.
public struct SoneError: Error, CustomStringConvertible {
    public enum Kind {
        case invalidArgument
        case ir
        case asset
        case render
    }

    public let kind: Kind
    public let message: String

    public var description: String { "\(kind): \(message)" }
}

/// Owns the font registry and the decoded-image cache.
///
/// Skia's font collection is shared inside an engine, so one engine renders one
/// document at a time and every call takes the lock. Give each thread its own
/// `Engine` for real parallelism rather than sharing one.
public final class Engine {
    private let handle: OpaquePointer
    private let lock = NSLock()
    private var closed = false

    /// - Parameter baseDir: the directory relative asset paths resolve against.
    public init(_ baseDir: String? = nil) {
        let directory = baseDir ?? FileManager.default.currentDirectoryPath
        guard let handle = directory.withCString({ Native.engineNew($0) }) else {
            fatalError("could not create a sone engine")
        }
        self.handle = handle
    }

    deinit {
        if !closed { Native.engineFree(handle) }
    }

    /// The process-wide engine, used when no explicit one is passed.
    public static let shared = Engine()

    /// The native library version.
    public static var version: String {
        Native.version().map { String(cString: $0) } ?? "unknown"
    }

    public func close() {
        lock.lock()
        defer { lock.unlock() }
        guard !closed else { return }
        closed = true
        Native.engineFree(handle)
    }

    // MARK: - fonts and assets

    /// Register a font family from raw TTF/OTF bytes.
    public func registerFont(_ name: String, _ data: Data) throws {
        try withBytes(data) { pointer, count in
            try check(name.withCString { Native.registerFont(try live(), $0, pointer, count) })
        }
    }

    /// Register a font family from a file.
    public func registerFontFile(_ name: String, _ path: String) throws {
        let handle = try live()
        try check(name.withCString { namePointer in
            path.withCString { pathPointer in
                Native.registerFontFile(handle, namePointer, pathPointer)
            }
        })
    }

    /// Make bytes available to documents as `asset:name`.
    public func registerImage(_ name: String, _ data: Data) throws {
        try withBytes(data) { pointer, count in
            try check(name.withCString { Native.registerImage(try live(), $0, pointer, count) })
        }
    }

    /// Whether a family has been registered.
    public func hasFont(_ name: String) throws -> Bool {
        let handle = try live()
        return name.withCString { Native.hasFont(handle, $0) }
    }

    /// Every registered family name.
    public func fontFamilies() throws -> [String] {
        let handle = try live()
        let json = try buffer { Native.fontFamilies(handle, $0) }
        let decoded = try? JSONSerialization.jsonObject(with: json)
        return decoded as? [String] ?? []
    }

    /// Drop every registered font.
    public func resetFonts() throws {
        Native.resetFonts(try live())
    }

    // MARK: - rendering

    /// Render an IR document to bytes.
    public func render(
        _ document: String,
        format: OutputFormat = .png,
        density: Double? = nil,
        quality: Double = 1.0,
        strict: Bool = false
    ) throws -> Data {
        let handle = try live()
        let options = self.options(format, density, quality, strict)
        return try buffer { out in
            document.withCString { Native.renderJson(handle, $0, options, out) }
        }
    }

    /// One raster image per page. Requires `pageHeight` in the document config.
    public func renderPages(
        _ document: String,
        format: OutputFormat = .png,
        density: Double? = nil,
        quality: Double = 1.0,
        strict: Bool = false
    ) throws -> [Data] {
        let handle = try live()
        let options = self.options(format, density, quality, strict)
        lock.lock()
        defer { lock.unlock() }

        var list = SoneBufferList()
        let status = document.withCString { Native.renderPages(handle, $0, options, &list) }
        defer { Native.bufferListFree(&list) }
        try check(status)

        var pages: [Data] = []
        pages.reserveCapacity(Int(list.len))
        for index in 0..<Int(list.len) {
            let page = list.items[index]
            pages.append(page.data == nil || page.len == 0
                ? Data()
                : Data(bytes: page.data, count: Int(page.len)))
        }
        return pages
    }

    /// The computed layout tree, as JSON.
    public func dumpLayout(_ document: String) throws -> String {
        let handle = try live()
        let bytes = try buffer { out in
            document.withCString { Native.dumpLayout(handle, $0, out) }
        }
        return String(decoding: bytes, as: UTF8.self)
    }

    /// Dataset-style metadata, as JSON.
    public func dumpMetadata(_ document: String, granularity: Granularity = .node) throws -> String {
        let handle = try live()
        let bytes = try buffer { out in
            document.withCString { documentPointer in
                granularity.rawValue.withCString { granularityPointer in
                    Native.dumpMetadata(handle, documentPointer, granularityPointer, out)
                }
            }
        }
        return String(decoding: bytes, as: UTF8.self)
    }

    // MARK: - internals

    private func live() throws -> OpaquePointer {
        if closed {
            throw SoneError(kind: .invalidArgument, message: "this engine has been closed")
        }
        return handle
    }

    private func options(_ format: OutputFormat, _ density: Double?, _ quality: Double, _ strict: Bool)
        -> SoneRenderOptions {
        var options = SoneRenderOptions()
        options.format = SoneFormat(UInt32(format.code))
        // Zero tells the engine to fall back to the document's own config.
        options.density = Float(density ?? 0)
        options.quality = Float(quality)
        options.strict = strict ? 1 : 0
        return options
    }

    private func withBytes(_ data: Data, _ body: (UnsafePointer<UInt8>?, Int) throws -> Void) rethrows {
        var copy = data
        if copy.isEmpty { copy = Data([0]) }
        try copy.withUnsafeBytes { raw in
            try body(raw.bindMemory(to: UInt8.self).baseAddress, data.count)
        }
    }

    private func buffer(_ call: (UnsafeMutablePointer<SoneBuffer>) -> SoneStatus) throws -> Data {
        lock.lock()
        defer { lock.unlock() }

        var out = SoneBuffer()
        let status = call(&out)
        defer { Native.bufferFree(&out) }
        try check(status)

        guard let data = out.data, out.len > 0 else { return Data() }
        return Data(bytes: data, count: Int(out.len))
    }

    private func check(_ status: SoneStatus) throws {
        guard status != SoneStatus_Ok else { return }
        let message = Native.lastError(handle).map { String(cString: $0) }
            ?? "sone failed with status \(status.rawValue)"
        let kind: SoneError.Kind = switch status {
        case SoneStatus_InvalidArgument: .invalidArgument
        case SoneStatus_IrError: .ir
        case SoneStatus_AssetError: .asset
        default: .render
        }
        throw SoneError(kind: kind, message: message)
    }
}

/// Font registration on the process-wide engine, for scripts that do not want
/// to own one. Skia carries no system fonts, so at least one family must be
/// registered before any text renders.
public enum Font {
    public static func load(_ name: String, _ path: String) throws {
        try Engine.shared.registerFontFile(name, path)
    }

    public static func load(_ name: String, data: Data) throws {
        try Engine.shared.registerFont(name, data)
    }

    public static func has(_ name: String) throws -> Bool {
        try Engine.shared.hasFont(name)
    }

    public static func families() throws -> [String] {
        try Engine.shared.fontFamilies()
    }

    public static func reset() throws {
        try Engine.shared.resetFonts()
    }
}
