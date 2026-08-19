//! C ABI for the sone engine.
//!
//! One opaque `SoneEngine` owns the font registry and asset cache, so there is
//! no global state. Every byte buffer the library returns must be released with
//! `sone_buffer_free`, and every string with `sone_string_free`.

use std::ffi::{c_char, c_int, CStr, CString};
use std::path::PathBuf;
use std::ptr;
use std::sync::Mutex;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
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

        let Some(json) = as_str(json) else {
            set_error(handle, "document JSON is not valid UTF-8");
            return SoneStatus::InvalidArgument;
        };

        let parsed = if options.strict != 0 {
            Document::from_json_strict(json)
        } else {
            Document::from_json(json)
        };
        let doc = match parsed {
            Ok(doc) => doc,
            Err(e) => {
                set_error(handle, e);
                return SoneStatus::IrError;
            }
        };

        let base_dir = handle.base_dir.lock().unwrap().clone();
        if let Err(e) = handle.engine.load_document_fonts(&doc, &base_dir) {
            set_error(handle, e);
            return SoneStatus::AssetError;
        }

        let render_options = RenderOptions {
            format: options.format.into(),
            density: if options.density > 0.0 {
                options.density
            } else {
                doc.config.density.unwrap_or(1.0)
            },
            quality: if options.quality > 0.0 {
                options.quality
            } else {
                1.0
            },
            strict: options.strict != 0,
            debug_layout: false,
            debug_text: false,
        };

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
                *handle.last_error.lock().unwrap() = None;
                SoneStatus::Ok
            }
            Err(e) => {
                let status = match e.exit_code() {
                    2 => SoneStatus::IrError,
                    3 => SoneStatus::AssetError,
                    _ => SoneStatus::RenderError,
                };
                set_error(handle, e);
                status
            }
        }
    })
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
