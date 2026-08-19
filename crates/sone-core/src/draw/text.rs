use kurbo::Rect;

use crate::css::color::{Color, BLACK};
use crate::ir::TextAlign;
use crate::layout::engine::{BoxLayout, Sides};
use crate::paint::*;
use crate::style::{CompiledNode, Paint as StylePaint, RunStyle, TextContent};
use crate::text::bidi::Dir;
use crate::text::paragraph::{Paragraph, Segment};

use super::DrawCtx;

#[derive(Debug, Clone)]
pub struct PlacedRun<'a> {
    pub segment: &'a Segment,
    /// Left edge of the segment box.
    pub x: f32,
    /// Alphabetic baseline.
    pub baseline_y: f32,
    /// Segment width, including any justification stretch.
    pub width: f32,
    pub added_space_width: f32,
    pub dir: Dir,
}

impl PlacedRun<'_> {
    /// Top edge of the segment box, used for highlights and metadata.
    pub fn top(&self) -> f32 {
        self.baseline_y - self.segment.height + self.segment.metrics.descent
    }
}

fn count_spaces(s: &str) -> usize {
    s.bytes().filter(|b| *b == b' ').count()
}

/// Edges remapped into the local frame of a rotated text box.
fn local_edges(orientation: u32, border: Sides, padding: Sides) -> (f32, f32, f32) {
    let (l, r, t) = match orientation {
        90 => (
            (border.top, padding.top),
            (border.bottom, padding.bottom),
            (border.right, padding.right),
        ),
        270 => (
            (border.bottom, padding.bottom),
            (border.top, padding.top),
            (border.left, padding.left),
        ),
        _ => (
            (border.left, padding.left),
            (border.right, padding.right),
            (border.top, padding.top),
        ),
    };
    (l.0 + l.1, r.0 + r.1, t.0 + t.1)
}

/// Position every segment of every paragraph. Shared by drawing and metadata.
pub fn place_runs<'a>(
    content: &TextContent,
    paragraphs: &'a [Paragraph],
    layout: &BoxLayout,
    x: f32,
    y: f32,
    orientation: u32,
    skip_width_expansion: bool,
) -> Vec<PlacedRun<'a>> {
    let block = &content.block;
    let rotated = matches!(orientation, 90 | 270);

    let (left_inset, right_inset, _top_inset) =
        local_edges(orientation, layout.border, layout.padding);
    let space_x = left_inset + right_inset;

    let content_left = if block.content_box {
        match orientation {
            90 => layout.padding.top,
            270 => layout.padding.bottom,
            _ => layout.padding.left,
        }
    } else {
        0.0
    };
    let content_top = if block.content_box {
        match orientation {
            90 => layout.padding.right,
            270 => layout.padding.left,
            _ => layout.padding.top,
        }
    } else {
        0.0
    };
    let border_left = match orientation {
        90 => layout.border.top,
        270 => layout.border.bottom,
        _ => layout.border.left,
    };
    let border_top = match orientation {
        90 => layout.border.right,
        270 => layout.border.left,
        _ => layout.border.top,
    };

    let left = content_left + border_left;
    let top = content_top + border_top;
    let align = block.align;

    let container_width = if rotated { layout.height } else { layout.width };

    let mut out = Vec::new();
    let mut paragraph_offset_y = 0.0f32;

    for paragraph in paragraphs {
        let para_width =
            if skip_width_expansion && block.text_wrap == Some(crate::ir::TextWrap::Balance) {
                paragraph.width
            } else {
                container_width.max(paragraph.width) - space_x
            };
        let is_rtl = paragraph.base_dir == Dir::Rtl;

        let mut offset_y = paragraph.offset_y + paragraph_offset_y;
        paragraph_offset_y += paragraph.height;

        for (i, line) in paragraph.lines.iter().enumerate() {
            let trailing = para_width - line.width;
            let added_space_width =
                if align == Some(TextAlign::Justify) && trailing > 0.0 && line.spaces_count > 0 {
                    trailing / line.spaces_count as f32
                } else {
                    0.0
                };

            let mut offset_x = if is_rtl {
                match align {
                    Some(TextAlign::Center) => (para_width + line.width) / 2.0,
                    Some(TextAlign::Left) => line.width,
                    _ => para_width,
                }
            } else {
                match align {
                    Some(TextAlign::Center) => (para_width - line.width) / 2.0,
                    Some(TextAlign::Right) => para_width - line.width,
                    _ => {
                        if i == 0 {
                            block.indent_size
                        } else {
                            block.hanging_indent_size
                        }
                    }
                }
            };

            for segment in &line.segments {
                let span_offset_y = segment.style.offset_y;
                let spaces = count_spaces(&segment.text);
                let mut width = segment.width;
                if align == Some(TextAlign::Justify) && spaces > 0 {
                    width += added_space_width * spaces as f32;
                }

                if is_rtl {
                    offset_x -= width;
                }
                let text_x = x + left + offset_x;
                let baseline_y = y + line.baseline + top + offset_y + span_offset_y;
                if !is_rtl {
                    offset_x += width;
                }

                let dir =
                    segment
                        .style
                        .text_dir
                        .unwrap_or(if is_rtl { Dir::Rtl } else { Dir::Ltr });
                out.push(PlacedRun {
                    segment,
                    x: text_x,
                    baseline_y,
                    width,
                    added_space_width: if spaces > 0 { added_space_width } else { 0.0 },
                    dir,
                });
            }

            offset_y += line.height;
        }
    }

    out
}

fn decoration_color(override_color: Option<Color>, color: &StylePaint) -> Color {
    override_color.unwrap_or(match color {
        StylePaint::Color(c) => *c,
        StylePaint::Gradient(_) => BLACK,
    })
}

pub fn draw_text_node<P: Painter>(
    p: &mut P,
    node: &CompiledNode,
    layout: &BoxLayout,
    x: f32,
    y: f32,
    ctx: &DrawCtx<'_>,
) {
    let Some(content) = node.text() else { return };
    let Some(text_layout) = ctx.state.text.get(&layout.index) else {
        return;
    };
    let orientation = content.block.orientation;

    if orientation == 0 {
        draw_blocks(p, content, &text_layout.paragraphs, layout, x, y, 0, ctx);
        return;
    }

    let w = layout.width;
    let h = layout.height;
    let depth = p.save();
    match orientation {
        90 => {
            p.translate(x + w, y);
            p.rotate_about(90.0, 0.0, 0.0);
        }
        180 => {
            p.translate(x + w, y + h);
            p.rotate_about(180.0, 0.0, 0.0);
        }
        270 => {
            p.translate(x, y + h);
            p.rotate_about(-90.0, 0.0, 0.0);
        }
        _ => {}
    }
    draw_blocks(
        p,
        content,
        &text_layout.paragraphs,
        layout,
        0.0,
        0.0,
        orientation,
        ctx,
    );
    p.restore_to_count(depth);
}

#[allow(clippy::too_many_arguments)]
fn draw_blocks<P: Painter>(
    p: &mut P,
    content: &TextContent,
    paragraphs: &[Paragraph],
    layout: &BoxLayout,
    x: f32,
    y: f32,
    orientation: u32,
    ctx: &DrawCtx<'_>,
) {
    let runs = place_runs(content, paragraphs, layout, x, y, orientation, false);

    for run in &runs {
        let style = &run.segment.style;
        let render_text = run
            .segment
            .tab_leader
            .as_deref()
            .unwrap_or(&run.segment.text);

        if style.underline != 0.0 {
            let thickness = style.size * 0.08;
            p.draw_rect(
                rect(run.x, run.baseline_y + thickness, run.width, thickness),
                &PaintSpec::fill(decoration_color(style.underline_color, &style.color)),
            );
        }

        if style.overline != 0.0 {
            let thickness = style.size * 0.08;
            p.draw_rect(
                rect(
                    run.x,
                    run.baseline_y - run.segment.metrics.ascent,
                    run.width,
                    thickness,
                ),
                &PaintSpec::fill(decoration_color(style.overline_color, &style.color)),
            );
        }

        if let Some(highlight) = style.highlight_color {
            p.draw_rect(
                rect(run.x, run.top(), run.width, run.segment.height),
                &PaintSpec::fill(highlight),
            );
        }

        if render_text.is_empty() {
            continue;
        }

        let mut draw_style = style.clone();
        if run.added_space_width > 0.0 {
            draw_style.word_spacing += run.added_space_width;
        }
        let shaping = draw_style.shaping();

        for shadow in &style.drop_shadows {
            let spec = crate::style::shadow_spec(shadow, crate::css::color::TRANSPARENT);
            p.draw_text_run(&TextRun {
                text: render_text,
                style: &shaping,
                x: run.x,
                baseline_y: run.baseline_y,
                width: run.width,
                direction: run.dir,
                fill: Some(PaintSpec::fill(spec.color)),
                stroke: None,
                drop_shadow: Some(spec),
            });
        }

        let stroke = match (style.stroke_color, style.stroke_width) {
            (Some(c), w) if w != 0.0 => {
                let mut paint = PaintSpec::stroke(c, w);
                if let Some(s) = &mut paint.stroke {
                    s.join = Join::Round;
                    s.miter_limit = 2.0;
                }
                Some(paint)
            }
            _ => None,
        };

        let fill = match &style.color {
            StylePaint::Gradient(gradients) => Some(PaintSpec::shader(Fill::Gradient {
                gradients: gradients.clone(),
                x,
                y,
                width: run.segment.width,
                height: run.segment.height,
            })),
            StylePaint::Color(c) => Some(PaintSpec::fill(*c)),
        };

        p.draw_text_run(&TextRun {
            text: render_text,
            style: &shaping,
            x: run.x,
            baseline_y: run.baseline_y,
            width: run.width,
            direction: run.dir,
            fill,
            stroke,
            drop_shadow: None,
        });

        if style.line_through != 0.0 {
            let thickness = style.size * 0.08;
            let m = run.segment.metrics;
            p.draw_rect(
                rect(
                    run.x,
                    run.baseline_y - (m.ascent - m.descent) / 2.0,
                    run.width,
                    thickness,
                ),
                &PaintSpec::fill(decoration_color(style.line_through_color, &style.color)),
            );
        }

        if ctx.debug_text {
            p.draw_rect(
                rect(run.x, run.top(), run.width, run.segment.height),
                &PaintSpec::stroke(crate::css::color::parse_color("cyan"), 2.0),
            );
        }
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(x as f64, y as f64, (x + w) as f64, (y + h) as f64)
}

/// Style used when a run has no segments of its own (empty paragraphs).
pub fn fallback_style(content: &TextContent) -> &RunStyle {
    &content.base
}
