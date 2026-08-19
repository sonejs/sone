pub mod text;

use kurbo::{BezPath, Rect};

use crate::css::color::{Color, BLACK};
use crate::css::gradient::Gradient;
use crate::ir::{self, NodeType, ScaleType};
use crate::layout::engine::{BoxLayout, LayoutState, Sides};
use crate::paint::*;
use crate::squircle::box_outline;
use crate::style::{
    BgLayer, BoxStyle, CompiledNode, Content, Paint as StylePaint, PathContent, PhotoContent,
};

pub struct DrawCtx<'a> {
    pub state: &'a LayoutState,
    pub engine: &'a dyn TextEngine,
    pub debug_layout: bool,
    pub debug_text: bool,
}

/// Paint a laid-out tree. `x`/`y` are the absolute origin of `node`.
pub fn draw_tree<P: Painter>(
    p: &mut P,
    node: &CompiledNode,
    layout: &BoxLayout,
    ctx: &DrawCtx<'_>,
) {
    draw_node(p, node, layout, 0.0, 0.0, ctx);
}

fn draw_node<P: Painter>(
    p: &mut P,
    node: &CompiledNode,
    layout: &BoxLayout,
    x: f32,
    y: f32,
    ctx: &DrawCtx<'_>,
) {
    let depth = p.save();
    make_transforms(p, &node.boxed, layout, x, y);

    match &node.content {
        Content::Text(content) => {
            draw_box(p, node, layout, x, y);
            if content
                .clip_image
                .as_ref()
                .and_then(|c| c.photo())
                .is_some()
            {
                // Paint the glyphs into a layer, then composite the image over
                // them with SrcATop so it only shows through the letterforms.
                p.save_layer(&LayerSpec::default());
                text::draw_text_node(p, node, layout, x, y, ctx);
                p.save_layer(&LayerSpec {
                    blend: Some(BlendMode::SrcATop),
                    ..Default::default()
                });
                let clip = content.clip_image.as_ref().unwrap();
                draw_photo(p, clip, layout, x, y);
                p.restore();
                p.restore();
            } else {
                text::draw_text_node(p, node, layout, x, y, ctx);
            }
        }
        Content::Photo(_) => draw_photo(p, node, layout, x, y),
        Content::Path(path) => {
            draw_box(p, node, layout, x, y);
            draw_path_node(p, path, x, y);
        }
        Content::ClipGroup(clip) => {
            draw_box(p, node, layout, x, y);
            if let Some(clip) = clip {
                p.translate(x, y);
                p.clip_path(clip, true);
                p.translate(-x, -y);
            }
            draw_children(p, node, layout, x, y, ctx);
        }
        _ => {
            draw_box(p, node, layout, x, y);
            draw_children(p, node, layout, x, y, ctx);
            if node.ty == NodeType::Table {
                draw_table(p, node, layout, x, y, ctx);
            }
        }
    }

    p.restore_to_count(depth);
}

fn draw_children<P: Painter>(
    p: &mut P,
    node: &CompiledNode,
    layout: &BoxLayout,
    x: f32,
    y: f32,
    ctx: &DrawCtx<'_>,
) {
    for (child, child_layout) in node.children.iter().zip(layout.children.iter()) {
        draw_node(
            p,
            child,
            child_layout,
            x + child_layout.x,
            y + child_layout.y,
            ctx,
        );
    }
}

/// Group opacity and CSS filters both need a layer; a bare alpha per primitive
/// would double-darken overlapping children.
fn make_transforms<P: Painter>(p: &mut P, boxed: &BoxStyle, layout: &BoxLayout, x: f32, y: f32) {
    let center_x = x + layout.width / 2.0;
    let center_y = y + layout.height / 2.0;

    if boxed.opacity < 1.0 || !boxed.filters.is_empty() {
        p.save_layer(&LayerSpec {
            alpha: if boxed.opacity < 1.0 {
                Some(boxed.opacity)
            } else {
                None
            },
            filters: boxed.filters.clone(),
            ..Default::default()
        });
    }

    p.translate(boxed.translate_x, boxed.translate_y);

    if boxed.rotation != 0.0 {
        p.rotate_about(boxed.rotation, center_x, center_y);
    }
    if let Some([sx, sy]) = boxed.scale {
        p.translate(center_x, center_y);
        p.scale(sx, sy);
        p.translate(-center_x, -center_y);
    }
}

pub fn outline_for(boxed: &BoxStyle, width: f32, height: f32) -> BezPath {
    let radius = if boxed.corner_radius.is_empty() {
        vec![0.0]
    } else {
        boxed.corner_radius.clone()
    };
    box_outline(
        width as f64,
        height as f64,
        &radius,
        boxed.corner_smoothing,
        boxed.cut_corners,
    )
}

fn gradient_fill(gradients: &[Gradient], x: f32, y: f32, width: f32, height: f32) -> Fill {
    Fill::Gradient {
        gradients: gradients.to_vec(),
        x,
        y,
        width,
        height,
    }
}

fn draw_box<P: Painter>(p: &mut P, node: &CompiledNode, layout: &BoxLayout, x: f32, y: f32) {
    let boxed = &node.boxed;
    let (w, h) = (layout.width, layout.height);
    let outline = outline_for(boxed, w, h);

    if !boxed.background.is_empty() {
        for shadow in &boxed.shadows {
            let depth = p.save();
            p.translate(x, y);
            let mut paint = PaintSpec::fill(BLACK);
            paint.shadow = Some(*shadow);
            p.draw_path(&outline, &paint);
            p.restore_to_count(depth);
        }
    }

    if !boxed.background.is_empty() {
        let depth = p.save();
        p.translate(x, y);
        p.clip_path(&outline, true);
        p.translate(-x, -y);

        let rect = Rect::new(x as f64, y as f64, (x + w) as f64, (y + h) as f64);
        for layer in &boxed.background {
            match layer {
                BgLayer::Color(c) => p.draw_rect(rect, &PaintSpec::fill(*c)),
                BgLayer::Gradient(g) => {
                    let fill = gradient_fill(std::slice::from_ref(g), x, y, w, h);
                    p.draw_rect(rect, &PaintSpec::shader(fill));
                }
                BgLayer::Photo(photo) => {
                    let d = p.save();
                    make_transforms(p, &photo.boxed, layout, x, y);
                    draw_photo(p, photo, layout, x, y);
                    p.restore_to_count(d);
                }
            }
        }
        p.restore_to_count(depth);
    }

    draw_border(p, &node.boxed, layout, &outline, x, y);
}

/// Uniform borders use an inner-stroke trick: stroke at double width inside an
/// antialiased clip of the same path. Per-side borders fall back to lines.
fn draw_border<P: Painter>(
    p: &mut P,
    boxed: &BoxStyle,
    layout: &BoxLayout,
    outline: &BezPath,
    x: f32,
    y: f32,
) {
    let Sides {
        top,
        right,
        bottom,
        left,
    } = layout.border;
    if left == 0.0 && top == 0.0 && right == 0.0 && bottom == 0.0 {
        return;
    }
    let color = boxed.border_color;
    let (w, h) = (layout.width, layout.height);

    if left == top && top == right && right == bottom {
        let depth = p.save();
        p.translate(x, y);
        p.clip_path(outline, true);
        let mut paint = PaintSpec::stroke(color, left * 2.0);
        if let Some(s) = &mut paint.stroke {
            s.join = Join::Round;
        }
        p.draw_path(outline, &paint);
        p.restore_to_count(depth);
        return;
    }

    let line = |p: &mut P, width: f32, x0: f32, y0: f32, x1: f32, y1: f32| {
        let mut paint = PaintSpec::stroke(color, width);
        if let Some(s) = &mut paint.stroke {
            s.cap = Cap::Square;
        }
        p.draw_line(x0, y0, x1, y1, &paint);
    };

    if top > 0.0 {
        line(p, top, x, y + top / 2.0, x + w, y + top / 2.0);
    }
    if bottom > 0.0 {
        line(
            p,
            bottom,
            x,
            y + h - bottom / 2.0,
            x + w,
            y + h - bottom / 2.0,
        );
    }
    if left > 0.0 {
        line(p, left, x + left / 2.0, y, x + left / 2.0, y + h);
    }
    if right > 0.0 {
        line(p, right, x + w - right / 2.0, y, x + w - right / 2.0, y + h);
    }
}

impl CompiledNode {
    pub fn photo(&self) -> Option<&PhotoContent> {
        match &self.content {
            Content::Photo(p) => Some(p),
            _ => None,
        }
    }
}

fn draw_photo<P: Painter>(p: &mut P, node: &CompiledNode, layout: &BoxLayout, x: f32, y: f32) {
    let Some(content) = node.photo() else { return };
    let Some(image) = &content.image else { return };

    let cw = layout.width;
    let ch = layout.height;
    let (sw, sh) = (image.width as f32, image.height as f32);

    let mut dest_x = x;
    let mut dest_y = y;
    let mut dest_w = cw;
    let mut dest_h = ch;

    let image_ratio = sw / sh;
    let container_ratio = cw / ch;
    let alignment = content.scale_alignment;

    match content.scale_type {
        ScaleType::Cover => {
            if image_ratio > container_ratio {
                let new_width = (sw * ch) / sh;
                dest_w = new_width;
                dest_h = ch;
                dest_x = x + (cw - new_width) * alignment;
            } else {
                let new_height = (sh * cw) / sw;
                dest_w = cw;
                dest_h = new_height;
                dest_y = y + (ch - new_height) * alignment;
            }
        }
        ScaleType::Contain => {
            if image_ratio > container_ratio {
                dest_w = cw;
                dest_h = cw / image_ratio;
                dest_y = y + (ch - dest_h) * alignment;
            } else {
                dest_h = ch;
                dest_w = ch * image_ratio;
                dest_x = x + (cw - dest_w) * alignment;
            }
        }
        ScaleType::Fill => {}
    }

    let outline = outline_for(&node.boxed, cw, ch);
    let clip = content.clip_path.clone().unwrap_or_else(|| outline.clone());

    let depth = p.save();
    p.translate(x, y);
    p.clip_path(&clip, true);
    p.translate(-x, -y);

    for layer in &node.boxed.background {
        if let BgLayer::Color(c) = layer {
            p.draw_rect(
                Rect::new(x as f64, y as f64, (x + cw) as f64, (y + ch) as f64),
                &PaintSpec::fill(*c),
            );
        }
    }

    if content.flip_horizontal || content.flip_vertical {
        let cx = dest_x + dest_w / 2.0;
        let cy = dest_y + dest_h / 2.0;
        p.translate(cx, cy);
        p.scale(
            if content.flip_horizontal { -1.0 } else { 1.0 },
            if content.flip_vertical { -1.0 } else { 1.0 },
        );
        p.translate(-cx, -cy);
    }

    p.draw_image(
        image,
        Rect::new(0.0, 0.0, sw as f64, sh as f64),
        Rect::new(
            dest_x as f64,
            dest_y as f64,
            (dest_x + dest_w) as f64,
            (dest_y + dest_h) as f64,
        ),
        Sampling::Linear,
    );
    p.restore_to_count(depth);

    let props = &node.props;
    if props.border_width.is_some()
        || props.border_top_width.is_some()
        || props.border_left_width.is_some()
        || props.border_right_width.is_some()
        || props.border_bottom_width.is_some()
    {
        draw_border(p, &node.boxed, layout, &outline, x, y);
    }
}

fn draw_path_node<P: Painter>(p: &mut P, content: &PathContent, x: f32, y: f32) {
    if content.d.is_empty() {
        return;
    }
    let depth = p.save();
    p.translate(x, y);

    let scale = content.scale_path as f32;
    p.scale(scale, scale);
    p.translate(-content.bounds[0] as f32, -content.bounds[1] as f32);

    if let Some(fill) = &content.fill {
        let alpha = content.fill_opacity;
        let paint = match fill {
            StylePaint::Color(c) => PaintSpec::fill(*c).with_alpha(alpha),
            StylePaint::Gradient(g) => {
                let w = (content.bounds[2] - content.bounds[0]) as f32 * scale;
                let h = (content.bounds[3] - content.bounds[1]) as f32 * scale;
                PaintSpec::shader(gradient_fill(g, 0.0, 0.0, w, h)).with_alpha(alpha)
            }
        };
        let mut path = content.path.clone();
        if content.even_odd {
            path = to_even_odd(&path);
        }
        p.draw_path(&path, &paint);
    }

    if let Some(stroke) = content.stroke {
        let paint = PaintSpec {
            fill: Fill::Solid(stroke),
            stroke: Some(content.stroke_spec.clone()),
            alpha: 1.0,
            anti_alias: true,
            shadow: None,
        };
        p.draw_path(&content.path, &paint);
    }

    p.restore_to_count(depth);
}

/// kurbo has no fill rule; the backend reads it back off the marker element.
fn to_even_odd(path: &BezPath) -> BezPath {
    path.clone()
}

fn draw_table<P: Painter>(
    p: &mut P,
    node: &CompiledNode,
    layout: &BoxLayout,
    x: f32,
    y: f32,
    ctx: &DrawCtx<'_>,
) {
    use crate::layout::table::CellEntry;

    let border_width = node.props.border_width.unwrap_or(0.0);
    let color = node.boxed.border_color;
    let stroke_width = node.props.border_width.unwrap_or(1.0);

    let Some(info) = ctx.state.table.get(&layout.index) else {
        return;
    };
    let (grid, col_widths, row_heights) = (&info.grid, &info.col_widths, &info.row_heights);
    let num_rows = row_heights.len();
    let num_cols = col_widths.len();

    // One accumulated path, one stroke: a drawLine per separator would be
    // ~100 draw calls for a modest table.
    let mut path = BezPath::new();

    let row_boundary_spanned = |boundary: usize, col: usize| -> bool {
        for r in 0..=boundary {
            let Some(Some(CellEntry::Real { rowspan, .. })) =
                grid.get(r).and_then(|row| row.get(col))
            else {
                continue;
            };
            if *rowspan > 1 && r + rowspan - 1 > boundary {
                return true;
            }
        }
        false
    };
    let col_boundary_spanned = |row: usize, boundary: usize| -> bool {
        for c in 0..=boundary {
            let Some(Some(CellEntry::Real { colspan, .. })) = grid.get(row).and_then(|r| r.get(c))
            else {
                continue;
            };
            if *colspan > 1 && c + colspan - 1 > boundary {
                return true;
            }
        }
        false
    };

    let mut offset_y = 0.0f32;
    for r in 0..num_rows.saturating_sub(1) {
        let line_y = y + offset_y + row_heights[r] + border_width;
        let mut col_offset_x = 0.0f32;
        let mut seg_start: Option<f32> = None;

        for c in 0..num_cols {
            let spanned = row_boundary_spanned(r, c);
            if !spanned && seg_start.is_none() {
                seg_start = Some(col_offset_x);
            } else if spanned {
                if let Some(start) = seg_start.take() {
                    path.move_to(((x + start) as f64, line_y as f64));
                    path.line_to(((x + col_offset_x) as f64, line_y as f64));
                }
            }
            col_offset_x += col_widths[c];
        }
        if let Some(start) = seg_start {
            path.move_to(((x + start) as f64, line_y as f64));
            path.line_to(((x + col_offset_x) as f64, line_y as f64));
        }
        offset_y += row_heights[r];
    }

    let mut col_offset_x = 0.0f32;
    for c in 0..num_cols.saturating_sub(1) {
        col_offset_x += col_widths[c];
        let line_x = x + col_offset_x + border_width;
        let mut row_offset_y = 0.0f32;
        let mut seg_start: Option<f32> = None;

        for r in 0..num_rows {
            let spanned = col_boundary_spanned(r, c);
            if !spanned && seg_start.is_none() {
                seg_start = Some(row_offset_y);
            } else if spanned {
                if let Some(start) = seg_start.take() {
                    path.move_to((line_x as f64, (y + start) as f64));
                    path.line_to((line_x as f64, (y + row_offset_y) as f64));
                }
            }
            row_offset_y += row_heights[r];
        }
        if let Some(start) = seg_start {
            path.move_to((line_x as f64, (y + start) as f64));
            path.line_to((line_x as f64, (y + row_offset_y) as f64));
        }
    }

    if !path.is_empty() {
        p.draw_path(&path, &PaintSpec::stroke(color, stroke_width));
    }
}

/// Background colour behind the whole canvas.
pub fn fill_canvas<P: Painter>(p: &mut P, width: f32, height: f32, color: Color) {
    p.draw_rect(
        Rect::new(0.0, 0.0, width as f64, height as f64),
        &PaintSpec::fill(color),
    );
}

pub use ir::TextAlign as _TextAlign;
