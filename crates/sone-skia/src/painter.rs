use kurbo::{BezPath, PathEl, Rect as KRect};
use skia_safe::{
    canvas::SaveLayerRec, image_filters, textlayout::TextAlign, textlayout::TextDirection,
    BlendMode as SkBlend, Canvas, ClipOp, Color4f, Paint, PaintCap, PaintJoin, PaintStyle, Path,
    PathBuilder, PathEffect, PathFillType, Point, Rect, SamplingOptions, Shader, TileMode,
};

use sone_core::css::color::Color;
use sone_core::css::filter::FilterOp;
use sone_core::css::gradient::{generate_gradient, normalize_stops, Gradient, GradientKind};
use sone_core::paint::*;
use sone_core::text::bidi::Dir;

use skia_safe::gradient::{
    shaders as gradient_shaders, Colors as SkColors, Gradient as SkGradient,
    Interpolation as GradientInterpolation,
};

use crate::image::as_skia;
use crate::text::{SkiaTextEngine, UNCONSTRAINED};

pub struct SkiaPainter<'a> {
    canvas: &'a Canvas,
    engine: &'a SkiaTextEngine,
}

impl<'a> SkiaPainter<'a> {
    pub fn new(canvas: &'a Canvas, engine: &'a SkiaTextEngine) -> Self {
        SkiaPainter { canvas, engine }
    }
}

fn color4f(c: Color) -> Color4f {
    Color4f::new(c.r(), c.g(), c.b(), c.a())
}

fn to_rect(r: KRect) -> Rect {
    Rect::new(r.x0 as f32, r.y0 as f32, r.x1 as f32, r.y1 as f32)
}

fn to_path(p: &BezPath) -> Path {
    build_path(p).detach()
}

fn build_path(p: &BezPath) -> PathBuilder {
    let mut path = PathBuilder::new();
    for el in p.elements() {
        match el {
            PathEl::MoveTo(p) => {
                path.move_to((p.x as f32, p.y as f32));
            }
            PathEl::LineTo(p) => {
                path.line_to((p.x as f32, p.y as f32));
            }
            PathEl::QuadTo(a, b) => {
                path.quad_to((a.x as f32, a.y as f32), (b.x as f32, b.y as f32));
            }
            PathEl::CurveTo(a, b, c) => {
                path.cubic_to(
                    (a.x as f32, a.y as f32),
                    (b.x as f32, b.y as f32),
                    (c.x as f32, c.y as f32),
                );
            }
            PathEl::ClosePath => {
                path.close();
            }
        }
    }
    path
}

fn shader_for(fill: &Fill) -> Option<Shader> {
    let Fill::Gradient {
        gradients,
        x,
        y,
        width,
        height,
    } = fill
    else {
        return None;
    };

    for g in gradients {
        match g.kind {
            GradientKind::Linear | GradientKind::RepeatingLinear => {
                let resolved =
                    generate_gradient(std::slice::from_ref(g), *width as f64, *height as f64);
                let Some(r) = resolved.first() else { continue };
                let stops = normalize_stops(&r.locations);
                let colors: Vec<Color4f> = r.colors.iter().map(|c| color4f(*c)).collect();
                if colors.is_empty() {
                    continue;
                }
                let start = Point::new(x + r.start.x as f32 * width, y + r.start.y as f32 * height);
                let end = Point::new(x + r.end.x as f32 * width, y + r.end.y as f32 * height);
                let gc = SkColors::new(
                    colors.as_slice(),
                    Some(stops.as_slice()),
                    TileMode::Clamp,
                    None,
                );
                let grad = SkGradient::new(gc, GradientInterpolation::default());
                return gradient_shaders::linear_gradient((start, end), &grad, None);
            }
            GradientKind::Radial | GradientKind::RepeatingRadial => {
                // Implemented here, unlike the TS engine, which parses and skips
                // radial gradients. Defaults to a farthest-corner circle.
                let (colors, stops) = radial_stops(g);
                if colors.is_empty() {
                    continue;
                }
                let center = Point::new(x + width / 2.0, y + height / 2.0);
                let radius = ((width * width + height * height).sqrt()) / 2.0;
                let mode = if g.kind == GradientKind::RepeatingRadial {
                    TileMode::Repeat
                } else {
                    TileMode::Clamp
                };
                let gc = SkColors::new(colors.as_slice(), Some(stops.as_slice()), mode, None);
                let grad = SkGradient::new(gc, GradientInterpolation::default());
                return gradient_shaders::radial_gradient((center, radius), &grad, None);
            }
            _ => continue,
        }
    }
    None
}

fn radial_stops(g: &Gradient) -> (Vec<Color4f>, Vec<f32>) {
    use sone_core::css::color::parse_color;
    use sone_core::css::gradient::StopLength;

    let n = g.stops.len();
    let mut colors = Vec::with_capacity(n);
    let mut positions = Vec::with_capacity(n);
    for (i, stop) in g.stops.iter().enumerate() {
        colors.push(color4f(parse_color(&stop.color)));
        positions.push(match &stop.length {
            Some(StopLength::Percent(v)) => *v / 100.0,
            Some(StopLength::Px(v)) => *v,
            _ => {
                if n <= 1 {
                    0.0
                } else {
                    i as f64 / (n - 1) as f64
                }
            }
        });
    }
    (colors, normalize_stops(&positions))
}

fn build_paint(spec: &PaintSpec) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(spec.anti_alias);

    match &spec.fill {
        Fill::Solid(c) => {
            paint.set_color4f(color4f(c.with_alpha(spec.alpha)), None);
        }
        gradient @ Fill::Gradient { .. } => {
            paint.set_shader(shader_for(gradient));
            paint.set_alpha_f(spec.alpha);
        }
    }

    if let Some(stroke) = &spec.stroke {
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(stroke.width);
        paint.set_stroke_cap(match stroke.cap {
            Cap::Butt => PaintCap::Butt,
            Cap::Round => PaintCap::Round,
            Cap::Square => PaintCap::Square,
        });
        paint.set_stroke_join(match stroke.join {
            Join::Miter => PaintJoin::Miter,
            Join::Round => PaintJoin::Round,
            Join::Bevel => PaintJoin::Bevel,
        });
        paint.set_stroke_miter(stroke.miter_limit);
        if let Some(dash) = &stroke.dash {
            if !dash.is_empty() {
                if let Some(effect) = PathEffect::dash(dash, stroke.dash_offset) {
                    paint.set_path_effect(effect);
                }
            }
        }
    } else {
        paint.set_style(PaintStyle::Fill);
    }

    if let Some(shadow) = &spec.shadow {
        paint.set_image_filter(image_filters::drop_shadow_only(
            (shadow.dx, shadow.dy),
            (shadow.sigma, shadow.sigma),
            color4f(shadow.color).to_color(),
            None,
            None,
            None,
        ));
    }

    paint
}

fn filter_chain(ops: &[FilterOp]) -> Option<skia_safe::ImageFilter> {
    let mut result: Option<skia_safe::ImageFilter> = None;
    for op in ops {
        let next = match op {
            FilterOp::Blur(sigma) => {
                image_filters::blur((*sigma as f32, *sigma as f32), TileMode::Decal, None, None)
            }
            FilterOp::ColorMatrix(m) => {
                let cf = skia_safe::color_filters::matrix_row_major(m, None);
                image_filters::color_filter(cf, None, None)
            }
            FilterOp::DropShadow {
                dx,
                dy,
                sigma,
                color,
            } => image_filters::drop_shadow(
                (*dx as f32, *dy as f32),
                (*sigma as f32, *sigma as f32),
                color4f(*color).to_color(),
                None,
                None,
                None,
            ),
        };
        let Some(next) = next else { continue };
        result = Some(match result {
            Some(prev) => {
                let fallback = next.clone();
                image_filters::compose(prev, next).unwrap_or(fallback)
            }
            None => next,
        });
    }
    result
}

impl Painter for SkiaPainter<'_> {
    fn save(&mut self) -> u32 {
        self.canvas.save() as u32
    }
    fn restore_to_count(&mut self, depth: u32) {
        self.canvas.restore_to_count(depth as usize);
    }
    fn restore(&mut self) {
        self.canvas.restore();
    }
    fn translate(&mut self, dx: f32, dy: f32) {
        self.canvas.translate((dx, dy));
    }
    fn scale(&mut self, sx: f32, sy: f32) {
        self.canvas.scale((sx, sy));
    }
    fn rotate_about(&mut self, degrees: f32, cx: f32, cy: f32) {
        self.canvas.rotate(degrees, Some(Point::new(cx, cy)));
    }
    fn concat(&mut self, m: &kurbo::Affine) {
        let c = m.as_coeffs();
        let matrix = skia_safe::Matrix::new_all(
            c[0] as f32,
            c[2] as f32,
            c[4] as f32,
            c[1] as f32,
            c[3] as f32,
            c[5] as f32,
            0.0,
            0.0,
            1.0,
        );
        self.canvas.concat(&matrix);
    }
    fn clip_rect(&mut self, r: KRect, anti_alias: bool) {
        self.canvas
            .clip_rect(to_rect(r), ClipOp::Intersect, anti_alias);
    }
    fn clip_path(&mut self, p: &BezPath, anti_alias: bool) {
        self.canvas
            .clip_path(&to_path(p), ClipOp::Intersect, anti_alias);
    }
    fn save_layer(&mut self, spec: &LayerSpec) {
        let filter = if spec.filters.is_empty() {
            None
        } else {
            filter_chain(&spec.filters)
        };
        let mut paint = Paint::default();
        if let Some(alpha) = spec.alpha {
            paint.set_alpha_f(alpha);
        }
        if let Some(blend) = spec.blend {
            paint.set_blend_mode(match blend {
                BlendMode::SrcOver => SkBlend::SrcOver,
                BlendMode::SrcATop => SkBlend::SrcATop,
            });
        }
        if let Some(f) = filter {
            // An image filter on the layer paint applies to the layer contents.
            paint.set_image_filter(f);
        }
        let rec = SaveLayerRec::default().paint(&paint);
        self.canvas.save_layer(&rec);
    }
    fn draw_rect(&mut self, r: KRect, paint: &PaintSpec) {
        self.canvas.draw_rect(to_rect(r), &build_paint(paint));
    }
    fn draw_path(&mut self, p: &BezPath, paint: &PaintSpec) {
        self.canvas.draw_path(&to_path(p), &build_paint(paint));
    }
    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, paint: &PaintSpec) {
        self.canvas
            .draw_line((x0, y0), (x1, y1), &build_paint(paint));
    }
    fn draw_image(&mut self, img: &ImageHandle, src: KRect, dst: KRect, sampling: Sampling) {
        let Some(image) = as_skia(img) else { return };
        let options = match sampling {
            Sampling::Nearest => SamplingOptions::default(),
            Sampling::Linear => SamplingOptions::from(skia_safe::FilterMode::Linear),
        };
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        self.canvas.draw_image_rect_with_sampling_options(
            image,
            Some((&to_rect(src), skia_safe::canvas::SrcRectConstraint::Fast)),
            to_rect(dst),
            options,
            &paint,
        );
    }
    fn draw_text_run(&mut self, run: &TextRun<'_>) {
        if run.text.is_empty() {
            return;
        }
        let ascent = self.engine.measure("", run.style).ascent;
        let top = run.baseline_y - ascent;

        let paint_run = |paint: Option<&PaintSpec>, shadow: Option<&DropShadowSpec>| {
            let mut ps = skia_safe::textlayout::ParagraphStyle::new();
            let mut ts = self.engine.text_style(run.style);
            if let Some(spec) = paint {
                ts.set_foreground_paint(&build_paint(spec));
            }
            ps.set_text_style(&ts);
            ps.set_text_direction(match run.direction {
                Dir::Rtl => TextDirection::RTL,
                Dir::Ltr => TextDirection::LTR,
            });
            // Always Left: sone has already anchored the segment box.
            ps.set_text_align(TextAlign::Left);

            let mut builder =
                skia_safe::textlayout::ParagraphBuilder::new(&ps, self.engine.fonts.collection());
            builder.add_text(run.text);
            let mut paragraph = builder.build();
            paragraph.layout(UNCONSTRAINED);

            if let Some(shadow) = shadow {
                let mut layer_paint = Paint::default();
                layer_paint.set_image_filter(image_filters::drop_shadow_only(
                    (shadow.dx, shadow.dy),
                    (shadow.sigma, shadow.sigma),
                    color4f(shadow.color).to_color(),
                    None,
                    None,
                    None,
                ));
                let rec = SaveLayerRec::default().paint(&layer_paint);
                self.canvas.save_layer(&rec);
                paragraph.paint(self.canvas, (run.x, top));
                self.canvas.restore();
            } else {
                paragraph.paint(self.canvas, (run.x, top));
            }
        };

        // Stroke then fill, matching strokeText-then-fillText order.
        if let Some(stroke) = &run.stroke {
            paint_run(Some(stroke), run.drop_shadow.as_ref());
        }
        if let Some(fill) = &run.fill {
            paint_run(
                Some(fill),
                if run.stroke.is_some() {
                    None
                } else {
                    run.drop_shadow.as_ref()
                },
            );
        }
    }
}

/// kurbo carries no fill rule, so even-odd paths are flagged at draw time.
pub fn with_even_odd(path: &BezPath) -> Path {
    let mut b = build_path(path);
    b.set_fill_type(PathFillType::EvenOdd);
    b.detach()
}
