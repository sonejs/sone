use std::any::Any;
use std::sync::Arc;

use kurbo::{Affine, BezPath, Rect};

use crate::css::color::Color;
use crate::css::filter::FilterOp;
use crate::css::gradient::Gradient;
use crate::text::bidi::Dir;

#[derive(Debug, Clone, PartialEq)]
pub enum Fill {
    Solid(Color),
    /// Linear gradients resolved against the shape's box.
    Gradient {
        gradients: Vec<Gradient>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrokeSpec {
    pub width: f32,
    pub cap: Cap,
    pub join: Join,
    pub miter_limit: f32,
    pub dash: Option<Vec<f32>>,
    pub dash_offset: f32,
}

impl Default for StrokeSpec {
    fn default() -> Self {
        StrokeSpec {
            width: 1.0,
            cap: Cap::Butt,
            join: Join::Miter,
            miter_limit: 4.0,
            dash: None,
            dash_offset: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaintSpec {
    pub fill: Fill,
    /// Present for stroked draws; absent for fills.
    pub stroke: Option<StrokeSpec>,
    /// Multiplied into the fill's alpha.
    pub alpha: f32,
    pub anti_alias: bool,
    /// Replaces the draw with its shadow — how box shadows are painted.
    pub shadow: Option<DropShadowSpec>,
}

impl PaintSpec {
    pub fn fill(color: Color) -> Self {
        PaintSpec {
            fill: Fill::Solid(color),
            stroke: None,
            alpha: 1.0,
            anti_alias: true,
            shadow: None,
        }
    }
    pub fn stroke(color: Color, width: f32) -> Self {
        PaintSpec {
            fill: Fill::Solid(color),
            stroke: Some(StrokeSpec {
                width,
                ..Default::default()
            }),
            alpha: 1.0,
            anti_alias: true,
            shadow: None,
        }
    }
    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }
    pub fn shader(fill: Fill) -> Self {
        PaintSpec {
            fill,
            stroke: None,
            alpha: 1.0,
            anti_alias: true,
            shadow: None,
        }
    }
}

/// A CSS drop shadow, already converted to a Gaussian sigma.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropShadowSpec {
    pub dx: f32,
    pub dy: f32,
    pub sigma: f32,
    pub color: Color,
    pub spread: f32,
    pub inset: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayerSpec {
    pub alpha: Option<f32>,
    pub filters: Vec<FilterOp>,
    /// Shadow-only filter — used for box shadows.
    pub drop_shadow_only: Option<DropShadowSpec>,
    pub blend: Option<BlendMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    SrcOver,
    SrcATop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sampling {
    Nearest,
    Linear,
}

/// Backend-owned decoded image.
#[derive(Clone)]
pub struct ImageHandle {
    pub width: u32,
    pub height: u32,
    pub inner: Arc<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for ImageHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageHandle")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// Width/ascent/descent — the entire measurement contract sone needs.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TextMetrics {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

impl TextMetrics {
    pub fn height(&self) -> f32 {
        self.ascent + self.descent
    }
}

/// Resolved shaping inputs for one run.
#[derive(Debug, Clone, PartialEq)]
pub struct SpanStyle {
    pub font: Vec<String>,
    pub size: f32,
    pub weight: i32,
    pub italic: bool,
    pub oblique: bool,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

impl Default for SpanStyle {
    fn default() -> Self {
        SpanStyle {
            font: vec!["sans-serif".into()],
            size: 11.0,
            weight: 400,
            italic: false,
            oblique: false,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        }
    }
}

impl SpanStyle {
    /// Cache key covering every shaping input.
    pub fn key(&self) -> String {
        format!(
            "{}|{}|{}|{}{}|{}|{}",
            self.font.join(","),
            self.size,
            self.weight,
            if self.italic { "i" } else { "" },
            if self.oblique { "o" } else { "" },
            self.letter_spacing,
            self.word_spacing
        )
    }
}

pub struct TextRun<'a> {
    pub text: &'a str,
    pub style: &'a SpanStyle,
    pub x: f32,
    pub baseline_y: f32,
    pub width: f32,
    pub direction: Dir,
    pub fill: Option<PaintSpec>,
    pub stroke: Option<PaintSpec>,
    /// Applied as a shadow-only layer around the run.
    pub drop_shadow: Option<DropShadowSpec>,
}

/// The drawing seam. Mirrors `src/skia/painter.ts`; no backend types leak.
pub trait Painter {
    fn save(&mut self) -> u32;
    fn restore_to_count(&mut self, depth: u32);
    fn translate(&mut self, dx: f32, dy: f32);
    fn scale(&mut self, sx: f32, sy: f32);
    fn rotate_about(&mut self, degrees: f32, cx: f32, cy: f32);
    fn concat(&mut self, m: &Affine);
    fn clip_rect(&mut self, r: Rect, anti_alias: bool);
    fn clip_path(&mut self, p: &BezPath, anti_alias: bool);
    fn save_layer(&mut self, spec: &LayerSpec);
    fn restore(&mut self);
    fn draw_rect(&mut self, r: Rect, paint: &PaintSpec);
    fn draw_path(&mut self, p: &BezPath, paint: &PaintSpec);
    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, paint: &PaintSpec);
    fn draw_image(&mut self, img: &ImageHandle, src: Rect, dst: Rect, sampling: Sampling);
    fn draw_text_run(&mut self, run: &TextRun<'_>);
}

pub trait TextEngine: Send + Sync {
    /// Width plus font-derived ascent/descent — never text-dependent verticals.
    fn measure(&self, text: &str, style: &SpanStyle) -> TextMetrics;
    /// Raw word-segment starts (byte offsets, ascending, starting at 0).
    fn word_starts(&self, text: &str) -> Vec<usize> {
        crate::text::linebreak::word_starts(text)
    }
    /// Grapheme cluster starts (byte offsets).
    fn grapheme_starts(&self, text: &str) -> Vec<usize> {
        crate::text::linebreak::grapheme_starts(text)
    }
    /// Word starts with sone's custom rules applied.
    fn break_points(&self, text: &str) -> Vec<usize> {
        crate::text::linebreak::apply_break_rules(text, &self.word_starts(text))
    }
    fn has_font(&self, family: &str) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Png,
    Jpeg,
    Webp,
    Raw,
    Pdf,
    Svg,
}

impl OutputFormat {
    pub fn from_extension(ext: &str) -> Option<OutputFormat> {
        Some(match ext.to_ascii_lowercase().as_str() {
            "png" => OutputFormat::Png,
            "jpg" | "jpeg" => OutputFormat::Jpeg,
            "webp" => OutputFormat::Webp,
            "raw" | "rgba" => OutputFormat::Raw,
            "pdf" => OutputFormat::Pdf,
            "svg" => OutputFormat::Svg,
            _ => return None,
        })
    }
}
