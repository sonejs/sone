//! Turning a tree into bytes.
//!
//! No FFI, no IR string: the builder already produced [`sone_core::ir::Node`],
//! so this hands it straight to the engine. It is the shortest path of any of
//! the bindings, because there is no boundary to cross.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use sone_core::ir::{self, Document, Node, RenderConfig};
use sone_core::paint::OutputFormat;
use sone_core::Result;
use sone_skia::render::{Engine as SkiaEngine, RenderOptions};
use sone_skia::Assets;

use crate::IntoNode;

/// The granularity of the boxes [`Rendering::metadata`] returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Granularity {
    #[default]
    Node,
    Line,
    Word,
}

impl Granularity {
    fn as_str(self) -> &'static str {
        match self {
            Granularity::Node => "node",
            Granularity::Line => "line",
            Granularity::Word => "word",
        }
    }
}

/// Owns the font registry and the decoded-image cache.
///
/// Skia's font collection is shared inside an engine, so one engine renders one
/// document at a time and every call takes the lock. Give each thread its own
/// engine for real parallelism rather than sharing one.
pub struct Engine {
    inner: Mutex<Inner>,
}

struct Inner {
    engine: SkiaEngine,
    assets: Assets,
    base_dir: PathBuf,
}

impl Engine {
    pub fn new(base_dir: impl Into<PathBuf>) -> Engine {
        let base_dir = base_dir.into();
        Engine {
            inner: Mutex::new(Inner {
                engine: SkiaEngine::new(),
                assets: Assets::new(base_dir.clone()),
                base_dir,
            }),
        }
    }

    /// The process-wide engine, used when no explicit one is passed.
    pub fn shared() -> &'static Engine {
        static SHARED: OnceLock<Engine> = OnceLock::new();
        SHARED.get_or_init(|| Engine::new(std::env::current_dir().unwrap_or_else(|_| ".".into())))
    }

    /// Register a font family from raw TTF/OTF bytes.
    pub fn register_font(&self, name: &str, data: Vec<u8>) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        inner.engine.text.fonts.register(name, data)?;
        inner.engine.text.clear_caches();
        Ok(())
    }

    /// Register a font family from a file.
    pub fn register_font_file(&self, name: &str, path: impl AsRef<Path>) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        inner.engine.load_font_file(name, path.as_ref())
    }

    /// Make bytes available to documents as `asset:<name>`.
    pub fn register_image(&self, name: &str, data: Vec<u8>) {
        let inner = self.inner.lock().unwrap();
        inner.assets.register(name, data);
    }

    /// Whether a family has been registered.
    pub fn has_font(&self, name: &str) -> bool {
        self.inner.lock().unwrap().engine.text.fonts.has(name)
    }

    /// Every registered family name.
    pub fn font_families(&self) -> Vec<String> {
        self.inner.lock().unwrap().engine.text.fonts.families()
    }

    /// Drop every registered font.
    pub fn reset_fonts(&self) {
        let inner = self.inner.lock().unwrap();
        inner.engine.text.fonts.reset();
        inner.engine.text.clear_caches();
    }

    fn render(&self, document: &Document, options: &RenderOptions) -> Result<Vec<u8>> {
        let inner = self.inner.lock().unwrap();
        inner.engine.load_document_fonts(document, &inner.base_dir)?;
        if options.format == OutputFormat::Pdf {
            return inner.engine.render_pdf(document, &inner.base_dir, options);
        }
        let prepared = inner.engine.prepare_with_assets(document, &inner.assets)?;
        inner.engine.encode(&prepared, options)
    }

    fn render_pages(&self, document: &Document, options: &RenderOptions) -> Result<Vec<Vec<u8>>> {
        let inner = self.inner.lock().unwrap();
        inner.engine.load_document_fonts(document, &inner.base_dir)?;
        inner.engine.render_pages(document, &inner.base_dir, options)
    }
}

/// Register a font on the process-wide engine.
///
/// Skia carries no system fonts, so at least one family must be registered
/// before any text renders.
pub fn font(name: &str, path: impl AsRef<Path>) -> Result<()> {
    Engine::shared().register_font_file(name, path)
}

/// A node plus its render configuration, with one method per output format.
pub struct Rendering<'a> {
    root: Node,
    config: RenderConfig,
    fonts: Vec<ir::FontSpec>,
    engine: Option<&'a Engine>,
    density: Option<f32>,
    quality: f32,
}

/// Wrap a node with render configuration.
pub fn render<'a>(root: impl IntoNode) -> Rendering<'a> {
    Rendering {
        root: root.into_node(),
        config: RenderConfig::default(),
        fonts: Vec::new(),
        engine: None,
        density: None,
        quality: 1.0,
    }
}

impl<'a> Rendering<'a> {
    /// Render on a specific engine rather than the process-wide one.
    pub fn engine(mut self, engine: &'a Engine) -> Self {
        self.engine = Some(engine);
        self
    }

    pub fn width(mut self, value: impl crate::value::Num) -> Self {
        self.config.width = Some(value.as_f32());
        self
    }

    pub fn height(mut self, value: impl crate::value::Num) -> Self {
        self.config.height = Some(value.as_f32());
        self
    }

    /// A CSS colour painted behind everything.
    pub fn background(mut self, value: impl Into<String>) -> Self {
        self.config.background = Some(value.into());
        self
    }

    /// Raster scale factor.
    pub fn density(mut self, value: impl crate::value::Num) -> Self {
        self.density = Some(value.as_f32());
        self.config.density = Some(value.as_f32());
        self
    }

    /// JPEG and WebP quality, 0..1.
    pub fn quality(mut self, value: impl crate::value::Num) -> Self {
        self.quality = value.as_f32();
        self
    }

    /// Turn the document into pages of this height.
    pub fn page_height(mut self, value: impl crate::value::Num) -> Self {
        self.config.page_height = Some(value.as_f32());
        self
    }

    /// Page margins, inside which the header, content and footer sit.
    pub fn margin(mut self, value: impl crate::value::Num) -> Self {
        let value = value.as_f32();
        self.config.margin = Some(ir::MarginSpec(ir::Margin {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }));
        self
    }

    pub fn margin_each(
        mut self,
        top: impl crate::value::Num,
        right: impl crate::value::Num,
        bottom: impl crate::value::Num,
        left: impl crate::value::Num,
    ) -> Self {
        self.config.margin = Some(ir::MarginSpec(ir::Margin {
            top: top.as_f32(),
            right: right.as_f32(),
            bottom: bottom.as_f32(),
            left: left.as_f32(),
        }));
        self
    }

    pub fn last_page_height(mut self, value: ir::LastPageHeight) -> Self {
        self.config.last_page_height = Some(value);
        self
    }

    /// Drawn at the top of every page. Use the literal tokens `{pageNumber}`
    /// and `{totalPages}` — the engine substitutes them.
    pub fn header(mut self, node: impl IntoNode) -> Self {
        self.config.header = Some(Box::new(node.into_node()));
        self
    }

    /// Drawn at the bottom of every page.
    pub fn footer(mut self, node: impl IntoNode) -> Self {
        self.config.footer = Some(Box::new(node.into_node()));
        self
    }

    /// A font the document carries with it, so another sone engine renders it
    /// identically.
    pub fn font(mut self, name: impl Into<String>, src: impl Into<String>) -> Self {
        self.fonts.push(ir::FontSpec {
            name: name.into(),
            src: src.into(),
        });
        self
    }

    // ── the document ────────────────────────────────────────────────────────

    /// The IR document.
    pub fn document(&self) -> Document {
        Document {
            sone: ir::IR_VERSION,
            config: self.config.clone(),
            fonts: self.fonts.clone(),
            root: self.root.clone(),
        }
    }

    /// The IR document as JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.document()).expect("the IR always serializes")
    }

    /// The IR document as indented JSON.
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.document()).expect("the IR always serializes")
    }

    // ── outputs ─────────────────────────────────────────────────────────────

    pub fn png(&self) -> Result<Vec<u8>> {
        self.encode(OutputFormat::Png)
    }

    pub fn jpeg(&self) -> Result<Vec<u8>> {
        self.encode(OutputFormat::Jpeg)
    }

    pub fn webp(&self) -> Result<Vec<u8>> {
        self.encode(OutputFormat::Webp)
    }

    /// Raw RGBA pixels, row-major, unpremultiplied.
    pub fn raw(&self) -> Result<Vec<u8>> {
        self.encode(OutputFormat::Raw)
    }

    /// A PDF. With a page height set, one page per break and selectable text.
    pub fn pdf(&self) -> Result<Vec<u8>> {
        self.encode(OutputFormat::Pdf)
    }

    pub fn svg(&self) -> Result<Vec<u8>> {
        self.encode(OutputFormat::Svg)
    }

    /// One raster image per page. Requires a page height.
    pub fn pages(&self) -> Result<Vec<Vec<u8>>> {
        let document = self.document();
        let options = self.options(OutputFormat::Png, &document);
        self.resolved_engine().render_pages(&document, &options)
    }

    /// Render and write to `path`, inferring the format from its extension.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(OutputFormat::from_extension)
            .ok_or_else(|| {
                sone_core::SoneError::Render(format!(
                    "cannot infer an output format from {}",
                    path.display()
                ))
            })?;
        let bytes = self.encode(format)?;
        std::fs::write(path, bytes)
            .map_err(|e| sone_core::SoneError::Render(format!("{}: {e}", path.display())))
    }

    // ── introspection ───────────────────────────────────────────────────────

    /// The computed layout tree.
    pub fn layout(&self) -> Result<serde_json::Value> {
        let document = self.document();
        let engine = self.resolved_engine();
        let inner = engine.inner.lock().unwrap();
        inner.engine.load_document_fonts(&document, &inner.base_dir)?;
        let prepared = inner.engine.prepare_with_assets(&document, &inner.assets)?;
        Ok(sone_core::dump::layout_json(&prepared.root, &prepared.layout))
    }

    /// Dataset-style boxes at node, line or word granularity.
    pub fn metadata(&self, granularity: Granularity) -> Result<serde_json::Value> {
        let document = self.document();
        let engine = self.resolved_engine();
        let inner = engine.inner.lock().unwrap();
        inner.engine.load_document_fonts(&document, &inner.base_dir)?;
        let prepared = inner.engine.prepare_with_assets(&document, &inner.assets)?;
        Ok(sone_core::metadata::build(
            &prepared.root,
            &prepared.layout,
            &prepared.state,
            granularity.as_str(),
        ))
    }

    /// Returns `&'a`, not `&self`: the borrow belongs to whoever supplied the
    /// engine, and the shared one is `'static`.
    fn resolved_engine(&self) -> &'a Engine {
        // A match rather than `unwrap_or_else`: passing the function item makes
        // the compiler unify 'a with 'static instead of coercing at the return.
        match self.engine {
            Some(engine) => engine,
            None => Engine::shared(),
        }
    }

    fn options(&self, format: OutputFormat, document: &Document) -> RenderOptions {
        RenderOptions {
            format,
            density: self.density.or(document.config.density).unwrap_or(1.0),
            quality: self.quality,
            strict: false,
            debug_layout: false,
            debug_text: false,
        }
    }

    fn encode(&self, format: OutputFormat) -> Result<Vec<u8>> {
        let document = self.document();
        let options = self.options(format, &document);
        self.resolved_engine().render(&document, &options)
    }
}
