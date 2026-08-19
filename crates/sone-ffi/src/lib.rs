//! C ABI for the sone engine.
//!
//! One opaque `SoneEngine` owns the font registry and asset cache, so there is
//! no global state. Every byte buffer the library returns must be released with
//! `sone_buffer_free`, and every buffer list with `sone_buffer_list_free`.
//!
//! Calls that produce text — `sone_font_families`, `sone_dump_layout`,
//! `sone_dump_metadata` — return UTF-8 JSON in a `SoneBuffer` rather than a
//! separate string type, so a binding has one ownership rule to learn.

use std::ffi::{c_char, c_int, CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Mutex;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
use sone_core::SoneError as CoreError;
use sone_skia::render::{Engine, RenderOptions};
use sone_skia::Assets;

/// Result codes. Mirrors the CLI's exit codes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoneStatus {
    /// The call succeeded.
    Ok = 0,
    /// A null pointer or invalid UTF-8 was passed in.
    InvalidArgument = 1,
    /// The IR document could not be parsed.
    IrError = 2,
    /// An image or font could not be loaded.
    AssetError = 3,
    /// Layout or rasterization failed.
    RenderError = 4,
}

/// Output formats accepted by `sone_render`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoneFormat {
    Png = 0,
    Jpeg = 1,
    Webp = 2,
    Raw = 3,
    Pdf = 4,
    Svg = 5,
}

impl From<SoneFormat> for OutputFormat {
    fn from(f: SoneFormat) -> OutputFormat {
        match f {
            SoneFormat::Png => OutputFormat::Png,
            SoneFormat::Jpeg => OutputFormat::Jpeg,
            SoneFormat::Webp => OutputFormat::Webp,
            SoneFormat::Raw => OutputFormat::Raw,
            SoneFormat::Pdf => OutputFormat::Pdf,
            SoneFormat::Svg => OutputFormat::Svg,
        }
    }
}

/// Render options. Zero-initialize and set what you need; `density` and
/// `quality` fall back to 1.0 when left at 0.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SoneRenderOptions {
    pub format: SoneFormat,
    /// Raster scale factor. 0 means "use the document's `config.density`".
    pub density: f32,
    /// JPEG/WebP quality, 0..1. 0 means 1.0.
    pub quality: f32,
    /// Non-zero rejects unknown IR fields.
    pub strict: c_int,
}

impl SoneRenderOptions {
    /// The engine-side options, filling `density` and `quality` from the
    /// document when they were left at 0.
    fn resolve(&self, doc: &Document) -> RenderOptions {
        RenderOptions {
            format: self.format.into(),
            density: if self.density > 0.0 {
                self.density
            } else {
                doc.config.density.unwrap_or(1.0)
            },
            quality: if self.quality > 0.0 {
                self.quality
            } else {
                1.0
            },
            strict: self.strict != 0,
            debug_layout: false,
            debug_text: false,
        }
    }
}

/// An owned byte buffer. Release with `sone_buffer_free`.
#[repr(C)]
#[derive(Debug)]
pub struct SoneBuffer {
    pub data: *mut u8,
    pub len: usize,
    capacity: usize,
}

impl SoneBuffer {
    fn empty() -> SoneBuffer {
        SoneBuffer {
            data: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
    fn from_vec(mut v: Vec<u8>) -> SoneBuffer {
        let buffer = SoneBuffer {
            data: v.as_mut_ptr(),
            len: v.len(),
            capacity: v.capacity(),
        };
        std::mem::forget(v);
        buffer
    }
}

/// A list of owned byte buffers — one per page. Release the whole list with
/// `sone_buffer_list_free`; the individual buffers must not be freed.
#[repr(C)]
#[derive(Debug)]
pub struct SoneBufferList {
    pub items: *mut SoneBuffer,
    pub len: usize,
    capacity: usize,
}

impl SoneBufferList {
    fn empty() -> SoneBufferList {
        SoneBufferList {
            items: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
    fn from_pages(pages: Vec<Vec<u8>>) -> SoneBufferList {
        let mut items: Vec<SoneBuffer> = pages.into_iter().map(SoneBuffer::from_vec).collect();
        let list = SoneBufferList {
            items: items.as_mut_ptr(),
            len: items.len(),
            capacity: items.capacity(),
        };
        std::mem::forget(items);
        list
    }
}

/// Opaque engine handle: owns the font registry and the asset cache.
pub struct SoneEngine {
    engine: Engine,
    assets: Assets,
    base_dir: Mutex<PathBuf>,
    last_error: Mutex<Option<CString>>,
}

fn set_error(handle: &SoneEngine, message: impl std::fmt::Display) {
    let text = CString::new(message.to_string()).unwrap_or_else(|_| CString::new("error").unwrap());
    *handle.last_error.lock().unwrap() = Some(text);
}

fn clear_error(handle: &SoneEngine) {
    *handle.last_error.lock().unwrap() = None;
}

/// The status for an engine failure, using the same split as the CLI's exit
/// codes so every binding maps one error class to one exception type.
fn status_for(e: &CoreError) -> SoneStatus {
    match e.exit_code() {
        2 => SoneStatus::IrError,
        3 => SoneStatus::AssetError,
        _ => SoneStatus::RenderError,
    }
}

/// Stops a Rust panic at the C boundary, where unwinding is undefined.
/// Release builds abort on panic, so this only bites in a debug build — but
/// that is exactly where a panic is most likely.
fn guard<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or(fallback)
}

unsafe fn as_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

/// Create an engine. `base_dir` is the directory relative asset paths resolve
/// against; pass NULL for the process working directory.
///
/// # Safety
/// `base_dir` must be NULL or a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sone_engine_new(base_dir: *const c_char) -> *mut SoneEngine {
    let dir = as_str(base_dir)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let handle = SoneEngine {
        engine: Engine::new(),
        assets: Assets::new(dir.clone()),
        base_dir: Mutex::new(dir),
        last_error: Mutex::new(None),
    };
    Box::into_raw(Box::new(handle))
}

/// Release an engine and everything it owns.
///
/// # Safety
/// `engine` must come from `sone_engine_new` and must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn sone_engine_free(engine: *mut SoneEngine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

/// The last error message, or NULL if the previous call succeeded. The string
/// stays valid until the next call on this engine.
///
/// # Safety
/// `engine` must be a live handle.
#[no_mangle]
pub unsafe extern "C" fn sone_engine_last_error(engine: *const SoneEngine) -> *const c_char {
    let Some(handle) = engine.as_ref() else {
        return ptr::null();
    };
    match handle.last_error.lock().unwrap().as_ref() {
        Some(text) => text.as_ptr(),
        None => ptr::null(),
    }
}

/// Register a font family from raw TTF/OTF bytes.
///
/// # Safety
/// `engine` must be live, `name` a valid UTF-8 C string, and `data`/`len` a
/// readable byte range.
#[no_mangle]
pub unsafe extern "C" fn sone_register_font(
    engine: *mut SoneEngine,
    name: *const c_char,
    data: *const u8,
    len: usize,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        let Some(handle) = engine.as_mut() else {
            return SoneStatus::InvalidArgument;
        };
        let Some(name) = as_str(name) else {
            set_error(handle, "font name is not valid UTF-8");
            return SoneStatus::InvalidArgument;
        };
        if data.is_null() {
            set_error(handle, "font data is null");
            return SoneStatus::InvalidArgument;
        }
        let bytes = std::slice::from_raw_parts(data, len).to_vec();
        match handle.engine.text.fonts.register(name, bytes) {
            Ok(()) => {
                handle.engine.text.clear_caches();
                *handle.last_error.lock().unwrap() = None;
                SoneStatus::Ok
            }
            Err(e) => {
                set_error(handle, e);
                SoneStatus::AssetError
            }
        }
    })
}

/// Register image bytes under `asset:<name>`, for callers with no filesystem.
///
/// # Safety
/// Same requirements as `sone_register_font`.
#[no_mangle]
pub unsafe extern "C" fn sone_register_image(
    engine: *mut SoneEngine,
    name: *const c_char,
    data: *const u8,
    len: usize,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        let Some(handle) = engine.as_mut() else {
            return SoneStatus::InvalidArgument;
        };
        let Some(name) = as_str(name) else {
            set_error(handle, "asset name is not valid UTF-8");
            return SoneStatus::InvalidArgument;
        };
        if data.is_null() {
            set_error(handle, "asset data is null");
            return SoneStatus::InvalidArgument;
        }
        handle
            .assets
            .register(name, std::slice::from_raw_parts(data, len).to_vec());
        *handle.last_error.lock().unwrap() = None;
        SoneStatus::Ok
    })
}

/// Render an IR document to bytes. On success `out` owns a buffer the caller
/// must release with `sone_buffer_free`.
///
/// # Safety
/// `engine` must be live, `json` a valid UTF-8 C string, and `out` writable.
#[no_mangle]
pub unsafe extern "C" fn sone_render_json(
    engine: *mut SoneEngine,
    json: *const c_char,
    options: SoneRenderOptions,
    out: *mut SoneBuffer,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        let Some(handle) = engine.as_mut() else {
            return SoneStatus::InvalidArgument;
        };
        if out.is_null() {
            set_error(handle, "output buffer pointer is null");
            return SoneStatus::InvalidArgument;
        }
        *out = SoneBuffer::empty();

        let doc = match document_for(handle, json, options.strict != 0) {
            Ok(doc) => doc,
            Err(status) => return status,
        };

        let base_dir = handle.base_dir.lock().unwrap().clone();
        let render_options = options.resolve(&doc);

        let result = if render_options.format == OutputFormat::Pdf {
            handle.engine.render_pdf(&doc, &base_dir, &render_options)
        } else {
            handle
                .engine
                .prepare_with_assets(&doc, &handle.assets)
                .and_then(|prepared| handle.engine.encode(&prepared, &render_options))
        };

        match result {
            Ok(bytes) => {
                *out = SoneBuffer::from_vec(bytes);
                clear_error(handle);
                SoneStatus::Ok
            }
            Err(e) => {
                let status = status_for(&e);
                set_error(handle, e);
                status
            }
        }
    })
}

/// Parse a document and register the fonts it declares, reporting any failure
/// on the handle. Every entry point that takes a document goes through here.
///
/// # Safety
/// `json` must be NULL or a valid NUL-terminated UTF-8 string.
unsafe fn document_for(
    handle: &SoneEngine,
    json: *const c_char,
    strict: bool,
) -> Result<Document, SoneStatus> {
    let Some(json) = as_str(json) else {
        set_error(handle, "document JSON is not valid UTF-8");
        return Err(SoneStatus::InvalidArgument);
    };
    let parsed = if strict {
        Document::from_json_strict(json)
    } else {
        Document::from_json(json)
    };
    let doc = parsed.map_err(|e| {
        set_error(handle, e);
        SoneStatus::IrError
    })?;
    let base_dir = handle.base_dir.lock().unwrap().clone();
    handle
        .engine
        .load_document_fonts(&doc, &base_dir)
        .map_err(|e| {
            set_error(handle, e);
            SoneStatus::AssetError
        })?;
    Ok(doc)
}

/// Lay a document out and hand the prepared tree to `f`, which produces the
/// JSON written into `out`. Shared by `sone_dump_layout` and
/// `sone_dump_metadata`.
///
/// # Safety
/// Same requirements as `sone_dump_layout`.
unsafe fn dump_json(
    engine: *mut SoneEngine,
    json: *const c_char,
    out: *mut SoneBuffer,
    f: impl FnOnce(&sone_skia::render::Prepared) -> String,
) -> SoneStatus {
    let Some(handle) = engine.as_ref() else {
        return SoneStatus::InvalidArgument;
    };
    let Some(out) = out.as_mut() else {
        set_error(handle, "output buffer pointer is null");
        return SoneStatus::InvalidArgument;
    };
    *out = SoneBuffer::empty();

    let doc = match document_for(handle, json, false) {
        Ok(doc) => doc,
        Err(status) => return status,
    };
    match handle.engine.prepare_with_assets(&doc, &handle.assets) {
        Ok(prepared) => {
            *out = SoneBuffer::from_vec(f(&prepared).into_bytes());
            clear_error(handle);
            SoneStatus::Ok
        }
        Err(e) => {
            let status = status_for(&e);
            set_error(handle, e);
            status
        }
    }
}

/// Register a font family from a file on disk. `path` is used as given, so
/// relative paths resolve against the process working directory, not the
/// engine's `base_dir`.
///
/// # Safety
/// `engine` must be live, and `name`/`path` valid NUL-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn sone_register_font_file(
    engine: *mut SoneEngine,
    name: *const c_char,
    path: *const c_char,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        let Some(handle) = engine.as_ref() else {
            return SoneStatus::InvalidArgument;
        };
        let (Some(name), Some(path)) = (as_str(name), as_str(path)) else {
            set_error(handle, "font name or path is not valid UTF-8");
            return SoneStatus::InvalidArgument;
        };
        match handle.engine.load_font_file(name, Path::new(path)) {
            Ok(()) => {
                clear_error(handle);
                SoneStatus::Ok
            }
            Err(e) => {
                set_error(handle, e);
                SoneStatus::AssetError
            }
        }
    })
}

/// Whether a family has been registered. False for a null handle or name.
///
/// # Safety
/// `engine` must be live and `name` a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn sone_has_font(engine: *const SoneEngine, name: *const c_char) -> bool {
    guard(false, || {
        let (Some(handle), Some(name)) = (engine.as_ref(), as_str(name)) else {
            return false;
        };
        handle.engine.text.fonts.has(name)
    })
}

/// Every registered family name, written to `out` as a UTF-8 JSON array of
/// strings. Release `out` with `sone_buffer_free`.
///
/// # Safety
/// `engine` must be live and `out` writable.
#[no_mangle]
pub unsafe extern "C" fn sone_font_families(
    engine: *const SoneEngine,
    out: *mut SoneBuffer,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        let Some(out) = out.as_mut() else {
            return SoneStatus::InvalidArgument;
        };
        *out = SoneBuffer::empty();
        let Some(handle) = engine.as_ref() else {
            return SoneStatus::InvalidArgument;
        };
        let families = serde_json::to_string(&handle.engine.text.fonts.families())
            .unwrap_or_else(|_| "[]".to_string());
        *out = SoneBuffer::from_vec(families.into_bytes());
        clear_error(handle);
        SoneStatus::Ok
    })
}

/// Drop every registered font and the shaping caches that depend on them.
///
/// # Safety
/// `engine` must be live.
#[no_mangle]
pub unsafe extern "C" fn sone_reset_fonts(engine: *mut SoneEngine) {
    guard((), || {
        if let Some(handle) = engine.as_ref() {
            handle.engine.text.fonts.reset();
            handle.engine.text.clear_caches();
            clear_error(handle);
        }
    })
}

/// Render one raster image per page. Requires `config.pageHeight` in the
/// document; without it the result is a single page. On success `out` owns a
/// list the caller must release with `sone_buffer_list_free`.
///
/// # Safety
/// `engine` must be live, `json` a valid UTF-8 C string, and `out` writable.
#[no_mangle]
pub unsafe extern "C" fn sone_render_pages(
    engine: *mut SoneEngine,
    json: *const c_char,
    options: SoneRenderOptions,
    out: *mut SoneBufferList,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        let Some(handle) = engine.as_ref() else {
            return SoneStatus::InvalidArgument;
        };
        let Some(out) = out.as_mut() else {
            set_error(handle, "output list pointer is null");
            return SoneStatus::InvalidArgument;
        };
        *out = SoneBufferList::empty();

        let doc = match document_for(handle, json, options.strict != 0) {
            Ok(doc) => doc,
            Err(status) => return status,
        };
        let base_dir = handle.base_dir.lock().unwrap().clone();
        let render_options = options.resolve(&doc);

        match handle.engine.render_pages(&doc, &base_dir, &render_options) {
            Ok(pages) => {
                *out = SoneBufferList::from_pages(pages);
                clear_error(handle);
                SoneStatus::Ok
            }
            Err(e) => {
                let status = status_for(&e);
                set_error(handle, e);
                status
            }
        }
    })
}

/// The computed layout tree, written to `out` as UTF-8 JSON. Release with
/// `sone_buffer_free`.
///
/// # Safety
/// `engine` must be live, `json` a valid UTF-8 C string, and `out` writable.
#[no_mangle]
pub unsafe extern "C" fn sone_dump_layout(
    engine: *mut SoneEngine,
    json: *const c_char,
    out: *mut SoneBuffer,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        dump_json(engine, json, out, |prepared| {
            sone_core::dump::layout_json(&prepared.root, &prepared.layout).to_string()
        })
    })
}

/// Dataset-style metadata, written to `out` as UTF-8 JSON. `granularity` is
/// `"node"`, `"line"` or `"word"`; NULL means `"node"`. Release with
/// `sone_buffer_free`.
///
/// # Safety
/// `engine` must be live, `json` a valid UTF-8 C string, `granularity` NULL or
/// a valid UTF-8 C string, and `out` writable.
#[no_mangle]
pub unsafe extern "C" fn sone_dump_metadata(
    engine: *mut SoneEngine,
    json: *const c_char,
    granularity: *const c_char,
    out: *mut SoneBuffer,
) -> SoneStatus {
    guard(SoneStatus::RenderError, || {
        let granularity = as_str(granularity).unwrap_or("node");
        dump_json(engine, json, out, |prepared| {
            sone_core::metadata::build(
                &prepared.root,
                &prepared.layout,
                &prepared.state,
                granularity,
            )
            .to_string()
        })
    })
}

/// Release a buffer list returned by the library, and every buffer in it.
///
/// # Safety
/// `list` must be a list this library produced and not already freed.
#[no_mangle]
pub unsafe extern "C" fn sone_buffer_list_free(list: *mut SoneBufferList) {
    let Some(list) = list.as_mut() else {
        return;
    };
    if !list.items.is_null() {
        // `SoneBuffer` has no `Drop` — dropping the outer Vec alone would leak
        // every page's bytes.
        let mut items = Vec::from_raw_parts(list.items, list.len, list.capacity);
        for buffer in items.iter_mut() {
            sone_buffer_free(buffer);
        }
    }
    *list = SoneBufferList::empty();
}

/// Release a buffer returned by the library.
///
/// # Safety
/// `buffer` must be a buffer this library produced and not already freed.
#[no_mangle]
pub unsafe extern "C" fn sone_buffer_free(buffer: *mut SoneBuffer) {
    let Some(buffer) = buffer.as_mut() else {
        return;
    };
    if !buffer.data.is_null() {
        drop(Vec::from_raw_parts(
            buffer.data,
            buffer.len,
            buffer.capacity,
        ));
    }
    *buffer = SoneBuffer::empty();
}

/// Library version, as a static NUL-terminated string.
#[no_mangle]
pub extern "C" fn sone_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r##"{"sone":1,"root":{"type":"column","props":{"width":16,"height":16,"background":["red"]}}}"##;

    /// Three blocks with an explicit break before the second and third. Plain
    /// containers are walked into rather than treated as atomic, so heights
    /// alone would not split this — the breaks have to be declared.
    const PAGED: &str = r##"{"sone":1,"config":{"width":40,"pageHeight":200},"root":{"type":"column","children":[
        {"type":"column","props":{"height":60,"background":["red"]}},
        {"type":"column","props":{"height":60,"background":["green"],"pageBreak":"before"}},
        {"type":"column","props":{"height":60,"background":["blue"],"pageBreak":"before"}}
    ]}}"##;

    const FONT: &str = "../../fixtures/font/GeistMono-Regular.ttf";

    fn options(format: SoneFormat) -> SoneRenderOptions {
        SoneRenderOptions {
            format,
            density: 1.0,
            quality: 1.0,
            strict: 0,
        }
    }

    #[test]
    fn renders_one_buffer_per_page() {
        unsafe {
            let engine = sone_engine_new(ptr::null());
            let json = CString::new(PAGED).unwrap();
            let mut pages = SoneBufferList::empty();
            let status =
                sone_render_pages(engine, json.as_ptr(), options(SoneFormat::Png), &mut pages);
            assert_eq!(status, SoneStatus::Ok);
            assert_eq!(pages.len, 3, "one page per declared break");
            for index in 0..pages.len {
                let page = &*pages.items.add(index);
                assert_eq!(
                    std::slice::from_raw_parts(page.data, 8),
                    b"\x89PNG\r\n\x1a\n"
                );
            }
            sone_buffer_list_free(&mut pages);
            assert!(pages.items.is_null());
            assert_eq!(pages.len, 0);
            sone_engine_free(engine);
        }
    }

    /// The one call where a wrong `capacity` would corrupt the allocator
    /// silently, so free the list twice and let Miri/ASan have something to
    /// catch if the layout is ever wrong.
    #[test]
    fn freeing_a_page_list_twice_is_safe() {
        unsafe {
            let engine = sone_engine_new(ptr::null());
            let json = CString::new(PAGED).unwrap();
            let mut pages = SoneBufferList::empty();
            sone_render_pages(engine, json.as_ptr(), options(SoneFormat::Png), &mut pages);
            sone_buffer_list_free(&mut pages);
            sone_buffer_list_free(&mut pages);
            sone_engine_free(engine);
        }
    }

    #[test]
    fn dumps_layout_and_metadata_as_json() {
        unsafe {
            let engine = sone_engine_new(ptr::null());
            let json = CString::new(DOC).unwrap();

            let mut out = SoneBuffer::empty();
            assert_eq!(
                sone_dump_layout(engine, json.as_ptr(), &mut out),
                SoneStatus::Ok
            );
            let text = std::str::from_utf8(std::slice::from_raw_parts(out.data, out.len)).unwrap();
            assert!(text.contains("\"width\":16.0"), "{text}");
            sone_buffer_free(&mut out);

            assert_eq!(
                sone_dump_metadata(engine, json.as_ptr(), ptr::null(), &mut out),
                SoneStatus::Ok
            );
            let text = std::str::from_utf8(std::slice::from_raw_parts(out.data, out.len)).unwrap();
            assert!(text.starts_with('{'), "{text}");
            sone_buffer_free(&mut out);

            sone_engine_free(engine);
        }
    }

    #[test]
    fn registers_a_font_from_a_file_and_lists_it() {
        unsafe {
            let engine = sone_engine_new(ptr::null());
            let name = CString::new("Geist Mono").unwrap();
            let path = CString::new(FONT).unwrap();

            assert!(!sone_has_font(engine, name.as_ptr()));
            assert_eq!(
                sone_register_font_file(engine, name.as_ptr(), path.as_ptr()),
                SoneStatus::Ok
            );
            assert!(sone_has_font(engine, name.as_ptr()));

            let mut out = SoneBuffer::empty();
            assert_eq!(sone_font_families(engine, &mut out), SoneStatus::Ok);
            let text = std::str::from_utf8(std::slice::from_raw_parts(out.data, out.len)).unwrap();
            assert!(text.contains("Geist Mono"), "{text}");
            sone_buffer_free(&mut out);

            sone_reset_fonts(engine);
            assert!(!sone_has_font(engine, name.as_ptr()));
            sone_engine_free(engine);
        }
    }

    #[test]
    fn a_missing_font_file_is_an_asset_error() {
        unsafe {
            let engine = sone_engine_new(ptr::null());
            let name = CString::new("Nope").unwrap();
            let path = CString::new("does/not/exist.ttf").unwrap();
            assert_eq!(
                sone_register_font_file(engine, name.as_ptr(), path.as_ptr()),
                SoneStatus::AssetError
            );
            assert!(!sone_engine_last_error(engine).is_null());
            sone_engine_free(engine);
        }
    }

    #[test]
    fn renders_a_png_through_the_c_abi() {
        unsafe {
            let engine = sone_engine_new(ptr::null());
            assert!(!engine.is_null());
            let json = CString::new(DOC).unwrap();
            let mut out = SoneBuffer::empty();
            let options = SoneRenderOptions {
                format: SoneFormat::Png,
                density: 1.0,
                quality: 1.0,
                strict: 0,
            };
            let status = sone_render_json(engine, json.as_ptr(), options, &mut out);
            assert_eq!(status, SoneStatus::Ok);
            assert!(out.len > 8);
            let header = std::slice::from_raw_parts(out.data, 8);
            assert_eq!(header, b"\x89PNG\r\n\x1a\n");
            sone_buffer_free(&mut out);
            assert!(out.data.is_null());
            sone_engine_free(engine);
        }
    }

    #[test]
    fn reports_ir_errors() {
        unsafe {
            let engine = sone_engine_new(ptr::null());
            let json = CString::new(r#"{"sone":99,"root":{"type":"column"}}"#).unwrap();
            let mut out = SoneBuffer::empty();
            let options = SoneRenderOptions {
                format: SoneFormat::Png,
                density: 0.0,
                quality: 0.0,
                strict: 0,
            };
            assert_eq!(
                sone_render_json(engine, json.as_ptr(), options, &mut out),
                SoneStatus::IrError
            );
            let message = CStr::from_ptr(sone_engine_last_error(engine))
                .to_string_lossy()
                .to_string();
            assert!(message.contains("unsupported IR version"), "{message}");
            sone_engine_free(engine);
        }
    }

    #[test]
    fn null_handles_are_rejected() {
        unsafe {
            let mut out = SoneBuffer::empty();
            let options = SoneRenderOptions {
                format: SoneFormat::Png,
                density: 0.0,
                quality: 0.0,
                strict: 0,
            };
            assert_eq!(
                sone_render_json(ptr::null_mut(), ptr::null(), options, &mut out),
                SoneStatus::InvalidArgument
            );
        }
    }
}
