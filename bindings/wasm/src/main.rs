//! The WebAssembly engine.
//!
//! Same contract as every other binding — document in, bytes out — over an ABI
//! shaped for JavaScript rather than for C. Two differences from
//! `crates/sone-ffi` earn their keep here:
//!
//! - **Nothing is returned by struct or written through an out-parameter.**
//!   Every call returns a scalar, and byte buffers are opaque handles read
//!   through `sone_wasm_buffer_ptr` / `_len`. JavaScript never has to know a
//!   Rust struct's layout, and there is no `malloc` dance to place an out-param.
//! - **Strings are pointer + length, not NUL-terminated.** JavaScript already
//!   knows how long its strings are, and a document containing a NUL byte
//!   should be an IR error rather than a truncation.
//!
//! Free every handle: buffers with `sone_wasm_buffer_free`, page lists with
//! `sone_wasm_pages_free`, engines with `sone_wasm_engine_free`.

use std::path::PathBuf;
use std::slice;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
use sone_core::SoneError as CoreError;
use sone_skia::render::{Engine, RenderOptions};
use sone_skia::Assets;

/// Result codes, matching `SoneError::exit_code()` and the C ABI's statuses so
/// every binding maps one error class to one exception type.
const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = 1;
const IR_ERROR: i32 = 2;
const ASSET_ERROR: i32 = 3;
const RENDER_ERROR: i32 = 4;

fn status_for(e: &CoreError) -> i32 {
    match e.exit_code() {
        2 => IR_ERROR,
        3 => ASSET_ERROR,
        _ => RENDER_ERROR,
    }
}

/// An owned byte buffer, handed to JavaScript as an opaque pointer.
pub struct Buffer(Vec<u8>);

impl Buffer {
    fn leak(bytes: Vec<u8>) -> *mut Buffer {
        Box::into_raw(Box::new(Buffer(bytes)))
    }
}

/// One buffer per page.
pub struct Pages(Vec<Buffer>);

/// The engine, its asset cache, and the last failure.
pub struct WasmEngine {
    engine: Engine,
    assets: Assets,
    base_dir: PathBuf,
    last_error: Option<String>,
}

impl WasmEngine {
    fn fail(&mut self, e: CoreError) -> i32 {
        let status = status_for(&e);
        self.last_error = Some(e.to_string());
        status
    }

    /// Parse, register the document's declared fonts, and resolve the options.
    fn prepare(
        &self,
        document: &str,
        format: OutputFormat,
        density: f32,
        quality: f32,
        strict: bool,
    ) -> Result<(Document, RenderOptions), CoreError> {
        let doc = if strict {
            Document::from_json_strict(document)
        } else {
            Document::from_json(document)
        }?;
        self.engine.load_document_fonts(&doc, &self.base_dir)?;
        let options = RenderOptions {
            format,
            density: if density > 0.0 {
                density
            } else {
                doc.config.density.unwrap_or(1.0)
            },
            quality: if quality > 0.0 { quality } else { 1.0 },
            strict,
            debug_layout: false,
            debug_text: false,
        };
        Ok((doc, options))
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Borrow a `ptr`/`len` pair as UTF-8. Empty is a valid empty string.
///
/// # Safety
/// `ptr` must be readable for `len` bytes, or `len` must be 0.
unsafe fn as_str<'a>(ptr: *const u8, len: usize) -> Option<&'a str> {
    if len == 0 {
        return Some("");
    }
    if ptr.is_null() {
        return None;
    }
    std::str::from_utf8(slice::from_raw_parts(ptr, len)).ok()
}

/// # Safety
/// `ptr` must be readable for `len` bytes, or `len` must be 0.
unsafe fn as_bytes<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
    if len == 0 {
        return Some(&[]);
    }
    if ptr.is_null() {
        return None;
    }
    Some(slice::from_raw_parts(ptr, len))
}

/// # Safety
/// `engine` must come from `sone_wasm_engine_new` and still be live.
unsafe fn engine<'a>(engine: *mut WasmEngine) -> Option<&'a mut WasmEngine> {
    engine.as_mut()
}

// ── memory ───────────────────────────────────────────────────────────────────

// JavaScript needs somewhere inside the module's linear memory to put a
// document, a font, or an image before calling in. Emscripten's `malloc` would
// do, but exporting it means overriding the `EXPORTED_FUNCTIONS` list rustc
// generates from the `#[no_mangle]` symbols — and then maintaining that list by
// hand forever. These two are `#[no_mangle]`, so they are exported for free.

/// The layout every allocation here uses. `Vec::with_capacity` would be the
/// obvious way to do this, but it is allowed to over-allocate, and freeing it
/// again needs the *capacity* rather than the length JavaScript hands back.
/// Going through `alloc` directly keeps the two sides describing the same block.
fn layout_for(len: usize) -> std::alloc::Layout {
    std::alloc::Layout::from_size_align(len, 1).expect("a byte layout is always valid")
}

/// Allocate `len` bytes. Returns null when `len` is 0 or the allocation fails.
#[no_mangle]
pub extern "C" fn sone_wasm_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    unsafe { std::alloc::alloc(layout_for(len)) }
}

/// Release an allocation from `sone_wasm_alloc`. `len` must be the same value.
///
/// # Safety
/// `ptr` must come from `sone_wasm_alloc` with this `len`, and not be reused.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len != 0 {
        std::alloc::dealloc(ptr, layout_for(len));
    }
}

// ── engine lifecycle ─────────────────────────────────────────────────────────

/// Create an engine. `base_dir` only matters for the `fonts` array of an IR
/// document; a browser has no filesystem, so it is fixed at `.`.
#[no_mangle]
pub extern "C" fn sone_wasm_engine_new() -> *mut WasmEngine {
    let base_dir = PathBuf::from(".");
    Box::into_raw(Box::new(WasmEngine {
        engine: Engine::new(),
        assets: Assets::new(base_dir.clone()),
        base_dir,
        last_error: None,
    }))
}

/// Release an engine and everything it owns.
///
/// # Safety
/// `handle` must come from `sone_wasm_engine_new` and not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_engine_free(handle: *mut WasmEngine) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// The last failure as a UTF-8 buffer, or null if the previous call succeeded.
/// The caller owns the buffer.
///
/// # Safety
/// `handle` must be live.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_last_error(handle: *mut WasmEngine) -> *mut Buffer {
    match engine(handle).and_then(|h| h.last_error.take()) {
        Some(message) => Buffer::leak(message.into_bytes()),
        None => std::ptr::null_mut(),
    }
}

// ── buffers ──────────────────────────────────────────────────────────────────

/// # Safety
/// `buffer` must be a live buffer this module produced.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_buffer_ptr(buffer: *mut Buffer) -> *const u8 {
    match buffer.as_ref() {
        Some(b) => b.0.as_ptr(),
        None => std::ptr::null(),
    }
}

/// # Safety
/// `buffer` must be a live buffer this module produced.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_buffer_len(buffer: *mut Buffer) -> usize {
    buffer.as_ref().map_or(0, |b| b.0.len())
}

/// # Safety
/// `buffer` must be a buffer this module produced and not already freed.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_buffer_free(buffer: *mut Buffer) {
    if !buffer.is_null() {
        drop(Box::from_raw(buffer));
    }
}

/// # Safety
/// `pages` must be a live list this module produced.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_pages_len(pages: *mut Pages) -> usize {
    pages.as_ref().map_or(0, |p| p.0.len())
}

/// The page at `index`, borrowed — it stays owned by the list, so read it and
/// then release the whole list with `sone_wasm_pages_free`.
///
/// # Safety
/// `pages` must be live and `index` within `sone_wasm_pages_len`.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_pages_item(pages: *mut Pages, index: usize) -> *mut Buffer {
    match pages.as_mut().and_then(|p| p.0.get_mut(index)) {
        Some(page) => page as *mut Buffer,
        None => std::ptr::null_mut(),
    }
}

/// # Safety
/// `pages` must be a list this module produced and not already freed. The
/// buffers it holds must not be freed individually.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_pages_free(pages: *mut Pages) {
    if !pages.is_null() {
        drop(Box::from_raw(pages));
    }
}

// ── fonts and assets ─────────────────────────────────────────────────────────

/// Register a font family from raw TTF/OTF bytes.
///
/// # Safety
/// Both pointer/length pairs must be readable.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_register_font(
    handle: *mut WasmEngine,
    name: *const u8,
    name_len: usize,
    data: *const u8,
    data_len: usize,
) -> i32 {
    let Some(handle) = engine(handle) else {
        return INVALID_ARGUMENT;
    };
    let (Some(name), Some(bytes)) = (as_str(name, name_len), as_bytes(data, data_len)) else {
        handle.last_error = Some("font name is not valid UTF-8, or data is null".into());
        return INVALID_ARGUMENT;
    };
    match handle.engine.text.fonts.register(name, bytes.to_vec()) {
        Ok(()) => {
            handle.engine.text.clear_caches();
            handle.last_error = None;
            OK
        }
        // A font that will not load is an asset failure, whatever `exit_code()`
        // reports for `SoneError::Font` — the C ABI has always said so too.
        Err(e) => {
            handle.last_error = Some(e.to_string());
            ASSET_ERROR
        }
    }
}

/// Drop one family and the shaping caches that depend on it.
///
/// # Safety
/// `name`/`name_len` must be readable.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_unregister_font(
    handle: *mut WasmEngine,
    name: *const u8,
    name_len: usize,
) -> i32 {
    let Some(handle) = engine(handle) else {
        return INVALID_ARGUMENT;
    };
    let Some(name) = as_str(name, name_len) else {
        return INVALID_ARGUMENT;
    };
    handle.engine.text.fonts.unregister(name);
    handle.engine.text.clear_caches();
    OK
}

/// Make bytes available to documents as `asset:<name>`.
///
/// # Safety
/// Both pointer/length pairs must be readable.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_register_image(
    handle: *mut WasmEngine,
    name: *const u8,
    name_len: usize,
    data: *const u8,
    data_len: usize,
) -> i32 {
    let Some(handle) = engine(handle) else {
        return INVALID_ARGUMENT;
    };
    let (Some(name), Some(bytes)) = (as_str(name, name_len), as_bytes(data, data_len)) else {
        return INVALID_ARGUMENT;
    };
    handle.assets.register(name, bytes.to_vec());
    handle.last_error = None;
    OK
}

/// 1 when the family is registered, 0 otherwise.
///
/// # Safety
/// `name`/`name_len` must be readable.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_has_font(
    handle: *mut WasmEngine,
    name: *const u8,
    name_len: usize,
) -> i32 {
    match (engine(handle), as_str(name, name_len)) {
        (Some(handle), Some(name)) if handle.engine.text.fonts.has(name) => 1,
        _ => 0,
    }
}

/// Every registered family name, as a UTF-8 JSON array. The caller owns it.
///
/// # Safety
/// `handle` must be live.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_font_families(handle: *mut WasmEngine) -> *mut Buffer {
    let Some(handle) = engine(handle) else {
        return std::ptr::null_mut();
    };
    let families = serde_json::to_string(&handle.engine.text.fonts.families())
        .unwrap_or_else(|_| "[]".to_string());
    Buffer::leak(families.into_bytes())
}

/// Drop every registered font and the caches that depend on them.
///
/// # Safety
/// `handle` must be live.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_reset_fonts(handle: *mut WasmEngine) {
    if let Some(handle) = engine(handle) {
        handle.engine.text.fonts.reset();
        handle.engine.text.clear_caches();
    }
}

// ── rendering ────────────────────────────────────────────────────────────────

/// Render to a single encoded buffer, or null on failure — call
/// `sone_wasm_last_error` for the reason. `density` and `quality` fall back to
/// the document's values when 0.
///
/// # Safety
/// Both pointer/length pairs must be readable.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_render(
    handle: *mut WasmEngine,
    document: *const u8,
    document_len: usize,
    format: *const u8,
    format_len: usize,
    density: f32,
    quality: f32,
    strict: i32,
) -> *mut Buffer {
    let Some(handle) = engine(handle) else {
        return std::ptr::null_mut();
    };
    let Some((document, format)) = request(handle, document, document_len, format, format_len)
    else {
        return std::ptr::null_mut();
    };

    let result = handle
        .prepare(&document, format, density, quality, strict != 0)
        .and_then(|(doc, options)| {
            if options.format == OutputFormat::Pdf {
                handle.engine.render_pdf(&doc, &handle.base_dir, &options)
            } else {
                let prepared = handle.engine.prepare_with_assets(&doc, &handle.assets)?;
                handle.engine.encode(&prepared, &options)
            }
        });

    match result {
        Ok(bytes) => {
            handle.last_error = None;
            Buffer::leak(bytes)
        }
        Err(e) => {
            handle.fail(e);
            std::ptr::null_mut()
        }
    }
}

/// One raster image per page, or null on failure.
///
/// # Safety
/// Both pointer/length pairs must be readable.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_render_pages(
    handle: *mut WasmEngine,
    document: *const u8,
    document_len: usize,
    format: *const u8,
    format_len: usize,
    density: f32,
    quality: f32,
    strict: i32,
) -> *mut Pages {
    let Some(handle) = engine(handle) else {
        return std::ptr::null_mut();
    };
    let Some((document, format)) = request(handle, document, document_len, format, format_len)
    else {
        return std::ptr::null_mut();
    };

    let result = handle
        .prepare(&document, format, density, quality, strict != 0)
        .and_then(|(doc, options)| handle.engine.render_pages(&doc, &handle.base_dir, &options));

    match result {
        Ok(pages) => {
            handle.last_error = None;
            Box::into_raw(Box::new(Pages(pages.into_iter().map(Buffer).collect())))
        }
        Err(e) => {
            handle.fail(e);
            std::ptr::null_mut()
        }
    }
}

/// The computed layout tree as JSON when `granularity_len` is 0, otherwise the
/// dataset metadata at that granularity. Null on failure.
///
/// # Safety
/// All pointer/length pairs must be readable.
#[no_mangle]
pub unsafe extern "C" fn sone_wasm_dump(
    handle: *mut WasmEngine,
    document: *const u8,
    document_len: usize,
    granularity: *const u8,
    granularity_len: usize,
) -> *mut Buffer {
    let Some(handle) = engine(handle) else {
        return std::ptr::null_mut();
    };
    let (Some(document), Some(granularity)) = (
        as_str(document, document_len),
        as_str(granularity, granularity_len),
    ) else {
        handle.last_error = Some("document or granularity is not valid UTF-8".into());
        return std::ptr::null_mut();
    };
    let document = document.to_string();
    let granularity = granularity.to_string();

    let result = handle
        .prepare(&document, OutputFormat::Png, 0.0, 1.0, false)
        .and_then(|(doc, _)| {
            let prepared = handle.engine.prepare_with_assets(&doc, &handle.assets)?;
            Ok(if granularity.is_empty() {
                sone_core::dump::layout_json(&prepared.root, &prepared.layout).to_string()
            } else {
                sone_core::metadata::build(
                    &prepared.root,
                    &prepared.layout,
                    &prepared.state,
                    &granularity,
                )
                .to_string()
            })
        });

    match result {
        Ok(json) => {
            handle.last_error = None;
            Buffer::leak(json.into_bytes())
        }
        Err(e) => {
            handle.fail(e);
            std::ptr::null_mut()
        }
    }
}

/// Validate and copy the two arguments every render call shares.
///
/// # Safety
/// Both pointer/length pairs must be readable.
unsafe fn request(
    handle: &mut WasmEngine,
    document: *const u8,
    document_len: usize,
    format: *const u8,
    format_len: usize,
) -> Option<(String, OutputFormat)> {
    let (Some(document), Some(format)) =
        (as_str(document, document_len), as_str(format, format_len))
    else {
        handle.last_error = Some("document or format is not valid UTF-8".into());
        return None;
    };
    let Some(parsed) = OutputFormat::from_extension(format) else {
        handle.last_error = Some(format!("unknown output format {format:?}"));
        return None;
    };
    Some((document.to_string(), parsed))
}

/// The engine version, as a UTF-8 buffer the caller owns.
#[no_mangle]
pub extern "C" fn sone_wasm_version() -> *mut Buffer {
    Buffer::leak(env!("CARGO_PKG_VERSION").as_bytes().to_vec())
}

fn main() {}
