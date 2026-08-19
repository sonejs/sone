//! Python extension module.
//!
//! Deliberately thin: the fluent builder API lives in `python/sone`, which
//! produces an IR document; everything here is document-in, bytes-out. The
//! engine is held behind a mutex so two Python threads cannot drive one Skia
//! font collection concurrently — create an `Engine` per thread for parallelism.

use std::path::PathBuf;
use std::sync::Mutex;

use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use sone_core::ir::Document;
use sone_core::paint::OutputFormat;
use sone_core::SoneError as CoreError;
use sone_skia::render::{Engine, RenderOptions};
use sone_skia::Assets;

create_exception!(_engine, SoneError, pyo3::exceptions::PyException, "Base class for sone failures.");
create_exception!(_engine, IrError, SoneError, "The IR document could not be parsed.");
create_exception!(_engine, AssetError, SoneError, "A font or image could not be loaded.");
create_exception!(_engine, RenderError, SoneError, "Layout or rasterization failed.");

fn to_py_err(e: CoreError) -> PyErr {
    let message = e.to_string();
    match e.exit_code() {
        2 => IrError::new_err(message),
        3 => AssetError::new_err(message),
        _ => RenderError::new_err(message),
    }
}

fn parse_format(name: &str) -> PyResult<OutputFormat> {
    OutputFormat::from_extension(name)
        .ok_or_else(|| PyValueError::new_err(format!("unknown output format {name:?}")))
}

struct Inner {
    engine: Engine,
    assets: Assets,
    base_dir: PathBuf,
}

/// Owns the font registry and the decoded-image cache.
#[pyclass(name = "Engine", module = "sone._engine")]
struct PyEngine {
    inner: Mutex<Inner>,
}

impl PyEngine {
    fn with<T>(&self, f: impl FnOnce(&mut Inner) -> Result<T, CoreError>) -> PyResult<T> {
        let mut inner = self.inner.lock().unwrap();
        f(&mut inner).map_err(to_py_err)
    }
}

#[pymethods]
impl PyEngine {
    #[new]
    #[pyo3(signature = (base_dir = None))]
    fn new(base_dir: Option<PathBuf>) -> Self {
        let base_dir = base_dir.unwrap_or_else(|| PathBuf::from("."));
        PyEngine {
            inner: Mutex::new(Inner {
                engine: Engine::new(),
                assets: Assets::new(base_dir.clone()),
                base_dir,
            }),
        }
    }

    /// Register a font family from raw TTF/OTF bytes.
    fn register_font(&self, name: &str, data: Vec<u8>) -> PyResult<()> {
        self.with(|inner| {
            inner.engine.text.fonts.register(name, data)?;
            inner.engine.text.clear_caches();
            Ok(())
        })
    }

    /// Register a font family from a file.
    fn register_font_file(&self, name: &str, path: PathBuf) -> PyResult<()> {
        self.with(|inner| inner.engine.load_font_file(name, &path))
    }

    /// Make bytes available to documents as `asset:<name>`.
    fn register_image(&self, name: &str, data: Vec<u8>) -> PyResult<()> {
        let inner = self.inner.lock().unwrap();
        inner.assets.register(name, data);
        Ok(())
    }

    fn has_font(&self, name: &str) -> bool {
        self.inner.lock().unwrap().engine.text.fonts.has(name)
    }

    fn font_families(&self) -> Vec<String> {
        self.inner.lock().unwrap().engine.text.fonts.families()
    }

    fn reset_fonts(&self) {
        let inner = self.inner.lock().unwrap();
        inner.engine.text.fonts.reset();
        inner.engine.text.clear_caches();
    }

    /// Render a document to bytes in `format`.
    #[pyo3(signature = (document, format = "png", density = None, quality = 1.0, strict = false))]
    fn render<'py>(
        &self,
        py: Python<'py>,
        document: &str,
        format: &str,
        density: Option<f32>,
        quality: f32,
        strict: bool,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let format = parse_format(format)?;
        let bytes = py.detach(|| {
            self.with(|inner| {
                let doc = parse(document, strict)?;
                inner.engine.load_document_fonts(&doc, &inner.base_dir)?;
                let options = options_for(&doc, format, density, quality, strict);
                if format == OutputFormat::Pdf {
                    inner.engine.render_pdf(&doc, &inner.base_dir, &options)
                } else {
                    let prepared = inner.engine.prepare_with_assets(&doc, &inner.assets)?;
                    inner.engine.encode(&prepared, &options)
                }
            })
        })?;
        Ok(PyBytes::new(py, &bytes))
    }

    /// One raster image per page. Requires `pageHeight` in the document config.
    #[pyo3(signature = (document, format = "png", density = None, quality = 1.0, strict = false))]
    fn render_pages<'py>(
        &self,
        py: Python<'py>,
        document: &str,
        format: &str,
        density: Option<f32>,
        quality: f32,
        strict: bool,
    ) -> PyResult<Vec<Bound<'py, PyBytes>>> {
        let format = parse_format(format)?;
        let pages = py.detach(|| {
            self.with(|inner| {
                let doc = parse(document, strict)?;
                inner.engine.load_document_fonts(&doc, &inner.base_dir)?;
                let options = options_for(&doc, format, density, quality, strict);
                inner.engine.render_pages(&doc, &inner.base_dir, &options)
            })
        })?;
        Ok(pages.iter().map(|p| PyBytes::new(py, p)).collect())
    }

    /// The computed layout tree, as a JSON string.
    fn dump_layout(&self, py: Python<'_>, document: &str) -> PyResult<String> {
        py.detach(|| {
            self.with(|inner| {
                let doc = parse(document, false)?;
                inner.engine.load_document_fonts(&doc, &inner.base_dir)?;
                let prepared = inner.engine.prepare_with_assets(&doc, &inner.assets)?;
                Ok(sone_core::dump::layout_json(&prepared.root, &prepared.layout).to_string())
            })
        })
    }

    /// Dataset-style metadata, as a JSON string.
    #[pyo3(signature = (document, granularity = "node"))]
    fn dump_metadata(&self, py: Python<'_>, document: &str, granularity: &str) -> PyResult<String> {
        py.detach(|| {
            self.with(|inner| {
                let doc = parse(document, false)?;
                inner.engine.load_document_fonts(&doc, &inner.base_dir)?;
                let prepared = inner.engine.prepare_with_assets(&doc, &inner.assets)?;
                Ok(sone_core::metadata::build(
                    &prepared.root,
                    &prepared.layout,
                    &prepared.state,
                    granularity,
                )
                .to_string())
            })
        })
    }
}

fn parse(document: &str, strict: bool) -> Result<Document, CoreError> {
    if strict {
        Document::from_json_strict(document)
    } else {
        Document::from_json(document)
    }
}

fn options_for(
    doc: &Document,
    format: OutputFormat,
    density: Option<f32>,
    quality: f32,
    strict: bool,
) -> RenderOptions {
    RenderOptions {
        format,
        density: density.or(doc.config.density).unwrap_or(1.0),
        quality,
        strict,
        debug_layout: false,
        debug_text: false,
    }
}

#[pymodule]
fn _engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyEngine>()?;
    m.add("SoneError", m.py().get_type::<SoneError>())?;
    m.add("IrError", m.py().get_type::<IrError>())?;
    m.add("AssetError", m.py().get_type::<AssetError>())?;
    m.add("RenderError", m.py().get_type::<RenderError>())?;
    Ok(())
}
