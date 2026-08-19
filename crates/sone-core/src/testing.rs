//! Mock backends so core logic is testable without Skia.

use std::cell::RefCell;

use kurbo::{Affine, BezPath, Rect};

use crate::paint::*;
use crate::text::linebreak::{apply_break_rules, word_starts};

/// Every glyph is `size` wide; ascent/descent are fixed fractions of `size`.
pub struct FixedMetricsEngine {
    pub advance: f32,
    pub ascent_ratio: f32,
    pub descent_ratio: f32,
    pub families: Vec<String>,
}

impl Default for FixedMetricsEngine {
    fn default() -> Self {
        FixedMetricsEngine {
            advance: 1.0,
            ascent_ratio: 0.8,
            descent_ratio: 0.2,
            families: vec!["sans-serif".into()],
        }
    }
}

impl TextEngine for FixedMetricsEngine {
    fn measure(&self, text: &str, style: &SpanStyle) -> TextMetrics {
        TextMetrics {
            width: text.chars().count() as f32 * style.size * self.advance
                + style.letter_spacing * text.chars().count() as f32,
            ascent: style.size * self.ascent_ratio,
            descent: style.size * self.descent_ratio,
        }
    }
    fn break_points(&self, text: &str) -> Vec<usize> {
        apply_break_rules(text, &word_starts(text))
    }
    fn has_font(&self, family: &str) -> bool {
        self.families.iter().any(|f| f == family)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Save,
    RestoreToCount(u32),
    Restore,
    Translate(f32, f32),
    Scale(f32, f32),
    Rotate(f32, f32, f32),
    Concat,
    ClipRect(Rect),
    ClipPath,
    SaveLayer(LayerSpec),
    DrawRect(Rect, PaintSpec),
    DrawPath(PaintSpec),
    DrawLine(f32, f32, f32, f32),
    DrawImage(Rect, Rect),
    DrawText {
        text: String,
        x: f32,
        baseline_y: f32,
    },
}

/// Records the draw-op sequence so ordering can be asserted.
#[derive(Default)]
pub struct RecordingPainter {
    pub ops: RefCell<Vec<Op>>,
    depth: RefCell<u32>,
}

impl RecordingPainter {
    pub fn ops(&self) -> Vec<Op> {
        self.ops.borrow().clone()
    }
    fn push(&self, op: Op) {
        self.ops.borrow_mut().push(op);
    }
}

impl Painter for RecordingPainter {
    fn save(&mut self) -> u32 {
        let mut d = self.depth.borrow_mut();
        *d += 1;
        let depth = *d;
        drop(d);
        self.push(Op::Save);
        depth
    }
    fn restore_to_count(&mut self, depth: u32) {
        *self.depth.borrow_mut() = depth.saturating_sub(1);
        self.push(Op::RestoreToCount(depth));
    }
    fn restore(&mut self) {
        self.push(Op::Restore);
    }
    fn translate(&mut self, dx: f32, dy: f32) {
        self.push(Op::Translate(dx, dy));
    }
    fn scale(&mut self, sx: f32, sy: f32) {
        self.push(Op::Scale(sx, sy));
    }
    fn rotate_about(&mut self, degrees: f32, cx: f32, cy: f32) {
        self.push(Op::Rotate(degrees, cx, cy));
    }
    fn concat(&mut self, _m: &Affine) {
        self.push(Op::Concat);
    }
    fn clip_rect(&mut self, r: Rect, _aa: bool) {
        self.push(Op::ClipRect(r));
    }
    fn clip_path(&mut self, _p: &BezPath, _aa: bool) {
        self.push(Op::ClipPath);
    }
    fn save_layer(&mut self, spec: &LayerSpec) {
        self.push(Op::SaveLayer(spec.clone()));
    }
    fn draw_rect(&mut self, r: Rect, paint: &PaintSpec) {
        self.push(Op::DrawRect(r, paint.clone()));
    }
    fn draw_path(&mut self, _p: &BezPath, paint: &PaintSpec) {
        self.push(Op::DrawPath(paint.clone()));
    }
    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, _paint: &PaintSpec) {
        self.push(Op::DrawLine(x0, y0, x1, y1));
    }
    fn draw_image(&mut self, _img: &ImageHandle, src: Rect, dst: Rect, _s: Sampling) {
        self.push(Op::DrawImage(src, dst));
    }
    fn draw_text_run(&mut self, run: &TextRun<'_>) {
        self.push(Op::DrawText {
            text: run.text.to_string(),
            x: run.x,
            baseline_y: run.baseline_y,
        });
    }
}
