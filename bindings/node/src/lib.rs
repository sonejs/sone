//! Node-API addon.
//!
//! Deliberately thin, exactly like `bindings/python/src/lib.rs`: the fluent
//! builder API lives in `ts/`, which produces an IR document, and everything
//! here is document-in, bytes-out.
//!
//! The engine is held behind a mutex so two JavaScript threads cannot drive one
//! Skia font collection concurrently — create an `Engine` per worker for
//! parallelism. Rendering runs on the libuv threadpool via `AsyncTask` rather
//! than on the main thread, because it is CPU-bound and the TypeScript API is
//! promise-returning anyway.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::{AsyncTask, Buffer};
use napi::{Env, Task};
use napi_derive::napi;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
use sone_core::SoneError as CoreError;
use sone_skia::render::{Engine as SkiaEngine, RenderOptions};
use sone_skia::Assets;

/// Failures cross the boundary with a machine-readable prefix, which
/// `ts/errors.ts` strips when it rethrows an `IrError`, `AssetError` or
/// `RenderError`. Node-API has no room for a custom error `code`: its `code` is
/// always the napi status, so the class has to travel in the message.
fn to_napi_err(e: CoreError) -> napi::Error {
    let class = match e.exit_code() {
        2 => "ir",
        3 => "asset",
        _ => "render",
    };
    napi::Error::from_reason(format!("sone:{class}: {e}"))
}

/// A font that will not load is an asset failure, whatever `exit_code()` says:
/// `SoneError::Font` reports 4, but the C ABI has always returned
/// `SoneStatus::AssetError` here and the class contract is per failure *class*,
/// not per error variant.
fn to_asset_err(e: CoreError) -> napi::Error {
    napi::Error::from_reason(format!("sone:asset: {e}"))
}

fn parse_format(name: &str) -> napi::Result<OutputFormat> {
    OutputFormat::from_extension(name).ok_or_else(|| {
        napi::Error::from_reason(format!("sone:render: unknown output format {name:?}"))
    })
}

struct Inner {
    engine: SkiaEngine,
    assets: Assets,
    base_dir: PathBuf,
}

impl Inner {
    /// Parse, register the document's declared fonts, and hand back the
    /// resolved render options. Every entry point below starts here.
    fn prepare(
        &self,
        document: &str,
        format: OutputFormat,
        density: Option<f64>,
        quality: f64,
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
            density: density
                .map(|d| d as f32)
                .or(doc.config.density)
                .unwrap_or(1.0),
            quality: quality as f32,
            strict,
            debug_layout: false,
            debug_text: false,
        };
        Ok((doc, options))
    }
}

/// What every task carries: the shared engine plus the document to run.
struct Job {
    inner: Arc<Mutex<Inner>>,
    document: String,
    format: OutputFormat,
    density: Option<f64>,
    quality: f64,
    strict: bool,
}

/// Render to a single encoded buffer.
pub struct RenderTask(Job);

impl Task for RenderTask {
    type Output = Vec<u8>;
    type JsValue = Buffer;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let job = &self.0;
        let inner = job.inner.lock().unwrap();
        let run = || -> Result<Vec<u8>, CoreError> {
            let (doc, options) = inner.prepare(
                &job.document,
                job.format,
                job.density,
                job.quality,
                job.strict,
            )?;
            if options.format == OutputFormat::Pdf {
                inner.engine.render_pdf(&doc, &inner.base_dir, &options)
            } else {
                let prepared = inner.engine.prepare_with_assets(&doc, &inner.assets)?;
                inner.engine.encode(&prepared, &options)
            }
        };
        run().map_err(to_napi_err)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output.into())
    }
}

/// One raster image per page.
pub struct RenderPagesTask(Job);

impl Task for RenderPagesTask {
    type Output = Vec<Vec<u8>>;
    type JsValue = Vec<Buffer>;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let job = &self.0;
        let inner = job.inner.lock().unwrap();
        let run = || -> Result<Vec<Vec<u8>>, CoreError> {
            let (doc, options) = inner.prepare(
                &job.document,
                job.format,
                job.density,
                job.quality,
                job.strict,
            )?;
            inner.engine.render_pages(&doc, &inner.base_dir, &options)
        };
        run().map_err(to_napi_err)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output.into_iter().map(Buffer::from).collect())
    }
}

/// Lay a document out and describe the result as JSON — the computed layout
/// tree or the dataset metadata, depending on `granularity`.
pub struct DumpTask {
    inner: Arc<Mutex<Inner>>,
    document: String,
    /// `None` dumps the layout tree; `Some(g)` dumps metadata at granularity `g`.
    granularity: Option<String>,
}

impl Task for DumpTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let inner = self.inner.lock().unwrap();
        let run = || -> Result<String, CoreError> {
            let (doc, _) = inner.prepare(&self.document, OutputFormat::Png, None, 1.0, false)?;
            let prepared = inner.engine.prepare_with_assets(&doc, &inner.assets)?;
            Ok(match &self.granularity {
                None => sone_core::dump::layout_json(&prepared.root, &prepared.layout).to_string(),
                Some(g) => {
                    sone_core::metadata::build(&prepared.root, &prepared.layout, &prepared.state, g)
                        .to_string()
                }
            })
        };
        run().map_err(to_napi_err)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

/// Owns the font registry and the decoded-image cache.
#[napi]
pub struct Engine {
    inner: Arc<Mutex<Inner>>,
}

#[napi]
impl Engine {
    /// `baseDir` is the directory relative asset paths resolve against.
    #[napi(constructor)]
    pub fn new(base_dir: Option<String>) -> Self {
        let base_dir = base_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Engine {
            inner: Arc::new(Mutex::new(Inner {
                engine: SkiaEngine::new(),
                assets: Assets::new(base_dir.clone()),
                base_dir,
            })),
        }
    }

    /// Register a font family from raw TTF/OTF bytes.
    #[napi]
    pub fn register_font(&self, name: String, data: Buffer) -> napi::Result<()> {
        let inner = self.inner.lock().unwrap();
        inner
            .engine
            .text
            .fonts
            .register(&name, data.to_vec())
            .map_err(to_asset_err)?;
        inner.engine.text.clear_caches();
        Ok(())
    }

    /// Register a font family from a file. The path is used as given, so a
    /// relative one resolves against the process working directory.
    #[napi]
    pub fn register_font_file(&self, name: String, path: String) -> napi::Result<()> {
        let inner = self.inner.lock().unwrap();
        inner
            .engine
            .load_font_file(&name, std::path::Path::new(&path))
            .map_err(to_asset_err)
    }

    /// Drop one family and the shaping caches that depend on it.
    #[napi]
    pub fn unregister_font(&self, name: String) {
        let inner = self.inner.lock().unwrap();
        inner.engine.text.fonts.unregister(&name);
        inner.engine.text.clear_caches();
    }

    /// Make bytes available to documents as `asset:<name>`.
    #[napi]
    pub fn register_image(&self, name: String, data: Buffer) {
        let inner = self.inner.lock().unwrap();
        inner.assets.register(&name, data.to_vec());
    }

    #[napi]
    pub fn has_font(&self, name: String) -> bool {
        self.inner.lock().unwrap().engine.text.fonts.has(&name)
    }

    #[napi]
    pub fn font_families(&self) -> Vec<String> {
        self.inner.lock().unwrap().engine.text.fonts.families()
    }

    #[napi]
    pub fn reset_fonts(&self) {
        let inner = self.inner.lock().unwrap();
        inner.engine.text.fonts.reset();
        inner.engine.text.clear_caches();
    }

    /// Render a document to bytes in `format`.
    #[napi(ts_return_type = "Promise<Buffer>")]
    pub fn render(
        &self,
        document: String,
        format: String,
        density: Option<f64>,
        quality: Option<f64>,
        strict: Option<bool>,
    ) -> napi::Result<AsyncTask<RenderTask>> {
        Ok(AsyncTask::new(RenderTask(
            self.job(document, format, density, quality, strict)?,
        )))
    }

    /// One raster image per page. Requires `pageHeight` in the document config.
    #[napi(ts_return_type = "Promise<Buffer[]>")]
    pub fn render_pages(
        &self,
        document: String,
        format: String,
        density: Option<f64>,
        quality: Option<f64>,
        strict: Option<bool>,
    ) -> napi::Result<AsyncTask<RenderPagesTask>> {
        Ok(AsyncTask::new(RenderPagesTask(
            self.job(document, format, density, quality, strict)?,
        )))
    }

    /// The computed layout tree, as a JSON string.
    #[napi(ts_return_type = "Promise<string>")]
    pub fn dump_layout(&self, document: String) -> AsyncTask<DumpTask> {
        AsyncTask::new(DumpTask {
            inner: Arc::clone(&self.inner),
            document,
            granularity: None,
        })
    }

    /// Dataset-style metadata, as a JSON string. `granularity` is `"node"`,
    /// `"line"` or `"word"`.
    #[napi(ts_return_type = "Promise<string>")]
    pub fn dump_metadata(
        &self,
        document: String,
        granularity: Option<String>,
    ) -> AsyncTask<DumpTask> {
        AsyncTask::new(DumpTask {
            inner: Arc::clone(&self.inner),
            document,
            granularity: Some(granularity.unwrap_or_else(|| "node".to_string())),
        })
    }

    fn job(
        &self,
        document: String,
        format: String,
        density: Option<f64>,
        quality: Option<f64>,
        strict: Option<bool>,
    ) -> napi::Result<Job> {
        Ok(Job {
            inner: Arc::clone(&self.inner),
            document,
            format: parse_format(&format)?,
            density,
            quality: quality.unwrap_or(1.0),
            strict: strict.unwrap_or(false),
        })
    }
}

/// The engine version, matching the Rust crates rather than the npm package.
#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
