import CSone
import Foundation

/// The C ABI from `include/sone.h`.
///
/// These are direct calls, not `dlsym` lookups. The library ships as a static
/// slice inside an XCFramework, and a static archive only contributes the
/// object files something references — resolving the symbols at runtime instead
/// would let the linker drop every one of them first.
enum Native {

    static func engineNew(_ baseDir: UnsafePointer<CChar>?) -> OpaquePointer? {
        sone_engine_new(baseDir)
    }

    static func engineFree(_ engine: OpaquePointer?) {
        sone_engine_free(engine)
    }

    static func lastError(_ engine: OpaquePointer?) -> UnsafePointer<CChar>? {
        sone_engine_last_error(engine)
    }

    static func registerFont(
        _ engine: OpaquePointer?, _ name: UnsafePointer<CChar>?,
        _ data: UnsafePointer<UInt8>?, _ len: Int
    ) -> SoneStatus {
        sone_register_font(engine, name, data, UInt(len))
    }

    static func registerFontFile(
        _ engine: OpaquePointer?, _ name: UnsafePointer<CChar>?, _ path: UnsafePointer<CChar>?
    ) -> SoneStatus {
        sone_register_font_file(engine, name, path)
    }

    static func registerImage(
        _ engine: OpaquePointer?, _ name: UnsafePointer<CChar>?,
        _ data: UnsafePointer<UInt8>?, _ len: Int
    ) -> SoneStatus {
        sone_register_image(engine, name, data, UInt(len))
    }

    static func hasFont(_ engine: OpaquePointer?, _ name: UnsafePointer<CChar>?) -> Bool {
        sone_has_font(engine, name)
    }

    static func fontFamilies(
        _ engine: OpaquePointer?, _ out: UnsafeMutablePointer<SoneBuffer>?
    ) -> SoneStatus {
        sone_font_families(engine, out)
    }

    static func resetFonts(_ engine: OpaquePointer?) {
        sone_reset_fonts(engine)
    }

    /// Options go over by pointer, never by value: struct-by-value is the one
    /// part of a C ABI that FFI layers disagree about.
    static func renderJson(
        _ engine: OpaquePointer?, _ json: UnsafePointer<CChar>?,
        _ options: UnsafePointer<SoneRenderOptions>?,
        _ out: UnsafeMutablePointer<SoneBuffer>?
    ) -> SoneStatus {
        sone_render_json(engine, json, options, out)
    }

    static func renderPages(
        _ engine: OpaquePointer?, _ json: UnsafePointer<CChar>?,
        _ options: UnsafePointer<SoneRenderOptions>?,
        _ out: UnsafeMutablePointer<SoneBufferList>?
    ) -> SoneStatus {
        sone_render_pages(engine, json, options, out)
    }

    static func dumpLayout(
        _ engine: OpaquePointer?, _ json: UnsafePointer<CChar>?,
        _ out: UnsafeMutablePointer<SoneBuffer>?
    ) -> SoneStatus {
        sone_dump_layout(engine, json, out)
    }

    static func dumpMetadata(
        _ engine: OpaquePointer?, _ json: UnsafePointer<CChar>?,
        _ granularity: UnsafePointer<CChar>?, _ out: UnsafeMutablePointer<SoneBuffer>?
    ) -> SoneStatus {
        sone_dump_metadata(engine, json, granularity, out)
    }

    static func bufferFree(_ buffer: UnsafeMutablePointer<SoneBuffer>?) {
        sone_buffer_free(buffer)
    }

    static func bufferListFree(_ list: UnsafeMutablePointer<SoneBufferList>?) {
        sone_buffer_list_free(list)
    }

    static func version() -> UnsafePointer<CChar>? {
        sone_version()
    }

    /// The repository root, when this package is used from a checkout. Only the
    /// tests need it — the engine itself resolves nothing at runtime any more.
    static let checkoutRoot: String? = {
        var directory = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)
        while true {
            let cargo = directory.appendingPathComponent("Cargo.toml")
            let crates = directory.appendingPathComponent("crates")
            if FileManager.default.fileExists(atPath: cargo.path),
               FileManager.default.fileExists(atPath: crates.path) {
                return directory.path
            }
            let parent = directory.deletingLastPathComponent()
            if parent.path == directory.path { return nil }
            directory = parent
        }
    }()
}
