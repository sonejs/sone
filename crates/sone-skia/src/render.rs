use std::path::{Path, PathBuf};

use skia_safe::{EncodedImageFormat, Rect};

use sone_core::compile::{compile, CompileCtx};
use sone_core::css::color::parse_color;
use sone_core::draw::{draw_tree, fill_canvas, DrawCtx};
use sone_core::error::SoneError;
use sone_core::ir;
use sone_core::ir::Document;
use sone_core::ir::Node;
use sone_core::layout::engine::{layout, BoxLayout, LayoutState};
use sone_core::pagination::{
    compute_page_breaks, has_page_tokens, substitute_page_tokens, Margins,
};
use sone_core::paint::{OutputFormat, Painter};
use sone_core::style::CompiledNode;
use sone_core::Result;

use crate::assets::Assets;
use crate::painter::SkiaPainter;
use crate::text::SkiaTextEngine;

pub struct Engine {
    pub text: SkiaTextEngine,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub format: OutputFormat,
    pub density: f32,
    /// JPEG/WebP quality, 0..1.
    pub quality: f32,
    pub strict: bool,
    pub debug_layout: bool,
    pub debug_text: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            format: OutputFormat::Png,
            density: 1.0,
            quality: 1.0,
            strict: false,
            debug_layout: false,
            debug_text: false,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

/// A laid-out document, ready to paint into any surface.
pub struct Prepared {
    pub root: CompiledNode,
    pub layout: BoxLayout,
    pub state: LayoutState,
    pub background: Option<String>,
    pub warnings: Vec<String>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            text: SkiaTextEngine::new(),
        }
    }

    pub fn load_font_file(&self, name: &str, path: &Path) -> Result<()> {
        let bytes = std::fs::read(path).map_err(|e| SoneError::Font {
            family: name.to_string(),
            message: format!("{}: {e}", path.display()),
        })?;
        self.text.fonts.register(name, bytes)?;
        self.text.clear_caches();
        Ok(())
    }

    /// Register every font the document declares, resolving paths against `base_dir`.
    pub fn load_document_fonts(&self, doc: &Document, base_dir: &Path) -> Result<()> {
        let assets = Assets::new(base_dir.to_path_buf());
        for font in &doc.fonts {
            let bytes = assets.read(&font.src)?;
            self.text.fonts.register(&font.name, bytes)?;
        }
        self.text.clear_caches();
        Ok(())
    }

    pub fn prepare(&self, doc: &Document, base_dir: &Path) -> Result<Prepared> {
        let assets = Assets::new(base_dir.to_path_buf());
        self.prepare_with_assets(doc, &assets)
    }

    pub fn prepare_with_assets(&self, doc: &Document, assets: &Assets) -> Result<Prepared> {
        let mut ctx = CompileCtx::new(assets);
        let root = compile(&doc.root, &mut ctx)?
            .ok_or_else(|| SoneError::Layout("the document root produced no layout".into()))?;
        let warnings = ctx.warnings().to_vec();
        let (layout_tree, state) = layout(&root, &self.text, doc.config.width, doc.config.height);
        Ok(Prepared {
            root,
            layout: layout_tree,
            state,
            background: doc.config.background.clone(),
            warnings,
        })
    }

    pub fn render(
        &self,
        doc: &Document,
        base_dir: &Path,
        options: &RenderOptions,
    ) -> Result<Vec<u8>> {
        let prepared = self.prepare(doc, base_dir)?;
        self.encode(&prepared, options)
    }

    fn paint(&self, prepared: &Prepared, canvas: &skia_safe::Canvas, options: &RenderOptions) {
        let mut painter = SkiaPainter::new(canvas, &self.text);
        let ctx = DrawCtx {
            state: &prepared.state,
            engine: &self.text,
            debug_layout: options.debug_layout,
            debug_text: options.debug_text,
        };
        if let Some(bg) = &prepared.background {
            fill_canvas(
                &mut painter,
                prepared.layout.width,
                prepared.layout.height,
                parse_color(bg),
            );
        }
        draw_tree(&mut painter, &prepared.root, &prepared.layout, &ctx);
    }

    pub fn encode(&self, prepared: &Prepared, options: &RenderOptions) -> Result<Vec<u8>> {
        let w = prepared.layout.width.max(1.0);
        let h = prepared.layout.height.max(1.0);

        match options.format {
            OutputFormat::Pdf => {
                let mut buffer: Vec<u8> = Vec::new();
                {
                    let mut document = skia_safe::pdf::new_document(&mut buffer, None);
                    let mut page = document.begin_page((w, h), None);
                    self.paint(prepared, page.canvas(), options);
                    document = page.end_page();
                    document.close();
                }
                Ok(buffer)
            }
            OutputFormat::Svg => {
                let canvas = skia_safe::svg::Canvas::new(Rect::from_wh(w, h), None);
                self.paint(prepared, &canvas, options);
                Ok(canvas.end().as_bytes().to_vec())
            }
            _ => {
                let density = options.density.max(0.01);
                // `ceil`, matching the TypeScript canvas exporter.
                let pw = (w * density).ceil().max(1.0) as i32;
                let ph = (h * density).ceil().max(1.0) as i32;
                let mut surface =
                    skia_safe::surfaces::raster_n32_premul((pw, ph)).ok_or_else(|| {
                        SoneError::Render(format!("could not allocate a {pw}x{ph} surface"))
                    })?;
                surface.canvas().scale((density, density));
                self.paint(prepared, surface.canvas(), options);
                encode_surface(&mut surface, options)
            }
        }
    }
}

fn encode_surface(surface: &mut skia_safe::Surface, options: &RenderOptions) -> Result<Vec<u8>> {
    let image = surface.image_snapshot();

    if options.format == OutputFormat::Raw {
        let info = skia_safe::ImageInfo::new(
            (image.width(), image.height()),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Unpremul,
            None,
        );
        let row_bytes = info.min_row_bytes();
        let mut pixels = vec![0u8; row_bytes * image.height() as usize];
        if !image.read_pixels(
            &info,
            &mut pixels,
            row_bytes,
            (0, 0),
            skia_safe::image::CachingHint::Allow,
        ) {
            return Err(SoneError::Render("could not read raw pixels back".into()));
        }
        return Ok(pixels);
    }

    let (format, quality) = match options.format {
        OutputFormat::Png => (EncodedImageFormat::PNG, 100),
        OutputFormat::Jpeg => (
            EncodedImageFormat::JPEG,
            (options.quality * 100.0).round() as u32,
        ),
        OutputFormat::Webp => (
            EncodedImageFormat::WEBP,
            (options.quality * 100.0).round() as u32,
        ),
        other => {
            return Err(SoneError::Render(format!(
                "{other:?} is not a raster format"
            )))
        }
    };

    let ctx: Option<&mut skia_safe::gpu::DirectContext> = None;
    let data = image
        .encode(ctx, format, quality)
        .ok_or_else(|| SoneError::Render(format!("{format:?} encoding failed")))?;
    Ok(data.as_bytes().to_vec())
}

/// Directory that relative asset paths resolve against.
pub fn base_dir_for(path: &Path) -> PathBuf {
    path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// One page of a paginated document: the content band plus its header/footer.
pub struct Page {
    pub width: f32,
    pub height: f32,
    content_offset_y: f32,
    content_offset_x: f32,
    clip: Option<(f32, f32)>,
    header: Option<(CompiledNode, BoxLayout, LayoutState)>,
    footer: Option<(CompiledNode, BoxLayout, LayoutState)>,
    header_height: f32,
    footer_height: f32,
    footer_y: f32,
}

impl Engine {
    /// Lay out a document into pages. Returns a single page when `pageHeight`
    /// is unset, so callers can treat both cases the same way.
    pub fn paginate(&self, doc: &Document, base_dir: &Path) -> Result<(Prepared, Vec<Page>)> {
        let prepared = self.prepare(doc, base_dir)?;
        let margins = Margins::from(doc.config.margin);
        let total_height = prepared.layout.height;
        let canvas_width = match doc.config.width {
            Some(w) => w,
            None => prepared.layout.width + margins.left + margins.right,
        };

        let Some(page_height) = doc.config.page_height else {
            return Ok((
                prepared,
                vec![Page {
                    width: canvas_width,
                    height: total_height,
                    content_offset_y: 0.0,
                    content_offset_x: 0.0,
                    clip: None,
                    header: None,
                    footer: None,
                    header_height: 0.0,
                    footer_height: 0.0,
                    footer_y: 0.0,
                }],
            ));
        };

        let assets = Assets::new(base_dir.to_path_buf());
        let band = |node: &Node| -> Result<(CompiledNode, BoxLayout, LayoutState)> {
            let mut ctx = CompileCtx::new(&assets);
            let compiled = compile(node, &mut ctx)?
                .ok_or_else(|| SoneError::Layout("header/footer produced no layout".into()))?;
            let (l, s) = layout(&compiled, &self.text, Some(canvas_width), None);
            Ok((compiled, l, s))
        };

        let header = doc.config.header.as_deref().map(band).transpose()?;
        let footer = doc.config.footer.as_deref().map(band).transpose()?;
        let header_height = header.as_ref().map(|(_, l, _)| l.height).unwrap_or(0.0);
        let footer_height = footer.as_ref().map(|(_, l, _)| l.height).unwrap_or(0.0);

        let content_page_height =
            page_height - header_height - footer_height - margins.top - margins.bottom;
        if content_page_height <= 0.0 {
            return Ok((prepared, Vec::new()));
        }

        let breaks = compute_page_breaks(
            &prepared.root,
            &prepared.layout,
            &prepared.state,
            content_page_height,
        );
        let mut starts = vec![0.0f32];
        starts.extend(breaks);
        let total_pages = starts.len();
        let uniform = doc.config.last_page_height != Some(ir::LastPageHeight::Content);
        let has_clip =
            header_height > 0.0 || footer_height > 0.0 || margins.top > 0.0 || margins.bottom > 0.0;

        let mut pages = Vec::with_capacity(total_pages);
        for (i, &start) in starts.iter().enumerate() {
            let next = starts.get(i + 1).copied().unwrap_or(total_height);
            let actual = next - start;
            let is_last = i == total_pages - 1;
            let content_h = if is_last && !uniform {
                actual
            } else {
                content_page_height
            };
            let canvas_height =
                header_height + margins.top + content_h + margins.bottom + footer_height;

            let page_number = i + 1;
            let stamp = |band: &Option<(CompiledNode, BoxLayout, LayoutState)>| {
                band.as_ref().map(|(node, l, _s)| {
                    if has_page_tokens(node) {
                        let stamped = substitute_page_tokens(node, page_number, total_pages);
                        let (nl, ns) = layout(&stamped, &self.text, Some(canvas_width), None);
                        (stamped, nl, ns)
                    } else {
                        (node.clone(), l.clone(), LayoutState::default())
                    }
                })
            };

            pages.push(Page {
                width: canvas_width,
                height: canvas_height,
                content_offset_y: header_height + margins.top - start,
                content_offset_x: margins.left,
                clip: if has_clip {
                    Some((header_height + margins.top, actual))
                } else {
                    None
                },
                header: stamp(&header),
                footer: stamp(&footer),
                header_height,
                footer_height,
                footer_y: header_height + margins.top + content_h + margins.bottom,
            });
        }

        // Bands without tokens are laid out once and shared, so reattach their
        // state to every page that reuses them.
        for page in &mut pages {
            if let (Some(src), Some(dst)) = (header.as_ref(), page.header.as_mut()) {
                if dst.2.text.is_empty() {
                    dst.2 = clone_state(&src.2);
                }
            }
            if let (Some(src), Some(dst)) = (footer.as_ref(), page.footer.as_mut()) {
                if dst.2.text.is_empty() {
                    dst.2 = clone_state(&src.2);
                }
            }
        }

        Ok((prepared, pages))
    }

    fn paint_page(
        &self,
        prepared: &Prepared,
        page: &Page,
        canvas: &skia_safe::Canvas,
        options: &RenderOptions,
    ) {
        let mut painter = SkiaPainter::new(canvas, &self.text);
        if let Some(bg) = &prepared.background {
            fill_canvas(&mut painter, page.width, page.height, parse_color(bg));
        }

        let ctx = DrawCtx {
            state: &prepared.state,
            engine: &self.text,
            debug_layout: options.debug_layout,
            debug_text: options.debug_text,
        };

        let depth = painter.save();
        if let Some((top, height)) = page.clip {
            painter.clip_rect(
                kurbo::Rect::new(0.0, top as f64, page.width as f64, (top + height) as f64),
                true,
            );
        }
        painter.translate(page.content_offset_x, page.content_offset_y);
        draw_tree(&mut painter, &prepared.root, &prepared.layout, &ctx);
        painter.restore_to_count(depth);

        if let Some((node, layout, state)) = &page.header {
            let ctx = DrawCtx {
                state,
                engine: &self.text,
                debug_layout: false,
                debug_text: false,
            };
            let depth = painter.save();
            painter.clip_rect(
                kurbo::Rect::new(0.0, 0.0, page.width as f64, page.header_height as f64),
                true,
            );
            draw_tree(&mut painter, node, layout, &ctx);
            painter.restore_to_count(depth);
        }

        if let Some((node, layout, state)) = &page.footer {
            let ctx = DrawCtx {
                state,
                engine: &self.text,
                debug_layout: false,
                debug_text: false,
            };
            let depth = painter.save();
            painter.clip_rect(
                kurbo::Rect::new(
                    0.0,
                    page.footer_y as f64,
                    page.width as f64,
                    (page.footer_y + page.footer_height) as f64,
                ),
                true,
            );
            painter.translate(0.0, page.footer_y);
            draw_tree(&mut painter, node, layout, &ctx);
            painter.restore_to_count(depth);
        }
    }

    /// Render every page into one PDF, keeping text selectable.
    pub fn render_pdf(
        &self,
        doc: &Document,
        base_dir: &Path,
        options: &RenderOptions,
    ) -> Result<Vec<u8>> {
        let (prepared, pages) = self.paginate(doc, base_dir)?;
        let mut buffer: Vec<u8> = Vec::new();
        {
            let mut document = skia_safe::pdf::new_document(&mut buffer, None);
            for page in &pages {
                let mut current = document.begin_page((page.width, page.height), None);
                self.paint_page(&prepared, page, current.canvas(), options);
                document = current.end_page();
            }
            document.close();
        }
        Ok(buffer)
    }

    /// Render every page as a separate raster image.
    pub fn render_pages(
        &self,
        doc: &Document,
        base_dir: &Path,
        options: &RenderOptions,
    ) -> Result<Vec<Vec<u8>>> {
        let (prepared, pages) = self.paginate(doc, base_dir)?;
        let density = options.density.max(0.01);
        let mut out = Vec::with_capacity(pages.len());
        for page in &pages {
            let pw = (page.width * density).ceil().max(1.0) as i32;
            let ph = (page.height * density).ceil().max(1.0) as i32;
            let mut surface =
                skia_safe::surfaces::raster_n32_premul((pw, ph)).ok_or_else(|| {
                    SoneError::Render(format!("could not allocate a {pw}x{ph} surface"))
                })?;
            surface.canvas().scale((density, density));
            self.paint_page(&prepared, page, surface.canvas(), options);
            out.push(encode_surface(&mut surface, options)?);
        }
        Ok(out)
    }
}

/// `LayoutState` holds no shared handles, so a page can take its own copy.
fn clone_state(state: &LayoutState) -> LayoutState {
    LayoutState {
        text: state.text.clone(),
        grid: state.grid.clone(),
        table: state.table.clone(),
    }
}
