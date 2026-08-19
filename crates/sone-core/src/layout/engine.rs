use std::collections::HashMap;

use taffy::geometry::Size;
use taffy::prelude::*;
use taffy::style::AvailableSpace;

use crate::ir::{self, NodeType};
use crate::paint::TextEngine;
use crate::style::{CompiledNode, Content};
use crate::text::paragraph::{create_paragraphs, Paragraph};

use super::grid::{resolve_grid, GridResolved};
use super::table::TableInfo;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Sides {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Default)]
pub struct BoxLayout {
    pub index: usize,
    pub id: u32,
    pub ty: Option<NodeType>,
    /// Offset within the parent's content box.
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub border: Sides,
    pub padding: Sides,
    pub margin: Sides,
    pub children: Vec<BoxLayout>,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    pub paragraphs: Vec<Paragraph>,
    /// Font size actually used, after autofit.
    pub size: f32,
}

#[derive(Default)]
pub struct LayoutState {
    pub text: HashMap<usize, TextLayout>,
    pub grid: HashMap<usize, GridResolved>,
    pub table: HashMap<usize, TableInfo>,
}

/// Flat, index-addressable view of the compiled tree.
pub struct Flat<'a> {
    pub nodes: Vec<&'a CompiledNode>,
    pub children: Vec<Vec<usize>>,
}

impl<'a> Flat<'a> {
    pub fn new(root: &'a CompiledNode) -> Self {
        let mut flat = Flat {
            nodes: Vec::new(),
            children: Vec::new(),
        };
        flat.push(root);
        flat
    }

    fn push(&mut self, node: &'a CompiledNode) -> usize {
        let index = self.nodes.len();
        self.nodes.push(node);
        self.children.push(Vec::new());
        let mut kids = Vec::with_capacity(node.children.len());
        for child in &node.children {
            kids.push(self.push(child));
        }
        self.children[index] = kids;
        index
    }
}

fn available(v: Option<f32>) -> AvailableSpace {
    match v {
        Some(v) => AvailableSpace::Definite(v),
        None => AvailableSpace::MaxContent,
    }
}

fn sides(r: taffy::geometry::Rect<f32>) -> Sides {
    Sides {
        top: r.top,
        right: r.right,
        bottom: r.bottom,
        left: r.left,
    }
}

/// Lay out one subtree in its own taffy tree, exactly as the TS engine creates
/// a fresh Yoga node per measurement root.
pub fn layout_subtree(
    flat: &Flat<'_>,
    index: usize,
    engine: &dyn TextEngine,
    state: &mut LayoutState,
    avail_w: Option<f32>,
    avail_h: Option<f32>,
    force_w: Option<f32>,
    force_h: Option<f32>,
) -> BoxLayout {
    let mut tree: TaffyTree<usize> = TaffyTree::new();
    // Yoga's pixel-grid rounding differs from taffy's (measured nodes round
    // outward), so it is applied in `collect` instead.
    tree.disable_rounding();
    let root = build_node(&mut tree, flat, index, engine, state);

    // Yoga sizes a layout root to the owner dimensions when it has no style
    // size of its own, so an available width is exact rather than a maximum.
    let node = flat.nodes[index];
    let root_w = force_w.or_else(|| {
        if node.props.width.is_none() && node.props.max_width.is_none() {
            avail_w
        } else {
            None
        }
    });
    let root_h = force_h.or_else(|| {
        if node.props.height.is_none() && node.props.max_height.is_none() {
            avail_h
        } else {
            None
        }
    });

    if let Some(w) = root_w {
        let mut s = tree.style(root).unwrap().clone();
        s.size.width = length(w);
        tree.set_style(root, s).unwrap();
    }
    if let Some(h) = root_h {
        let mut s = tree.style(root).unwrap().clone();
        s.size.height = length(h);
        tree.set_style(root, s).unwrap();
    }

    let space = Size {
        width: available(avail_w),
        height: available(avail_h),
    };
    tree.compute_layout_with_measure(root, space, |known, avail, _id, ctx, style| {
        let Some(&mut node_index) = ctx else {
            return Size::ZERO;
        };
        measure(
            flat,
            node_index,
            engine,
            state,
            known,
            avail,
            inset_of(style),
        )
    })
    .expect("taffy layout");

    collect(&tree, flat, root, index, state, 0.0, 0.0)
}

fn build_node(
    tree: &mut TaffyTree<usize>,
    flat: &Flat<'_>,
    index: usize,
    engine: &dyn TextEngine,
    state: &mut LayoutState,
) -> NodeId {
    let node = flat.nodes[index];
    let mut style = super::style::taffy_style(&node.props);

    match &node.content {
        Content::Text(_) | Content::Grid => {
            return tree.new_leaf_with_context(style, index).unwrap();
        }
        Content::Photo(_) => {
            return tree.new_leaf_with_context(style, index).unwrap();
        }
        Content::Path(path) => {
            // A path with no explicit size takes its scaled bounding box.
            let scale = path.scale_path as f32;
            if node.props.width.is_none() {
                style.size.width = length((path.bounds[2] - path.bounds[0]) as f32 * scale);
            }
            if node.props.height.is_none() {
                style.size.height = length((path.bounds[3] - path.bounds[1]) as f32 * scale);
            }
            return tree.new_leaf_with_context(style, index).unwrap();
        }
        _ => {}
    }

    let child_ids: Vec<NodeId> = flat.children[index]
        .iter()
        .map(|&c| build_node(tree, flat, c, engine, state))
        .collect();

    let id = tree.new_with_children(style, &child_ids).unwrap();
    tree.set_node_context(id, Some(index)).unwrap();

    if node.ty == NodeType::Table {
        super::table::apply_table_layout(tree, flat, index, &child_ids, engine, state);
    }

    id
}

/// Padding + border along each axis, in pixels. Percentages resolve to zero,
/// as they cannot be resolved without the containing block here.
fn inset_of(style: &taffy::Style) -> Size<f32> {
    let px = |v: taffy::style::LengthPercentage| {
        let raw = v.into_raw();
        if raw.tag() == taffy::style::CompactLength::LENGTH_TAG {
            raw.value()
        } else {
            0.0
        }
    };
    Size {
        width: px(style.padding.left)
            + px(style.padding.right)
            + px(style.border.left)
            + px(style.border.right),
        height: px(style.padding.top)
            + px(style.padding.bottom)
            + px(style.border.top)
            + px(style.border.bottom),
    }
}

fn measure(
    flat: &Flat<'_>,
    index: usize,
    engine: &dyn TextEngine,
    state: &mut LayoutState,
    known: Size<Option<f32>>,
    avail: Size<AvailableSpace>,
    inset: Size<f32>,
) -> Size<f32> {
    let node = flat.nodes[index];
    match &node.content {
        Content::Text(_) => measure_text(flat, index, engine, state, known, avail, inset),
        Content::Grid => {
            let resolved = resolve_grid(flat, index, engine, state, known, avail, inset);
            let size = Size {
                width: resolved.width,
                height: resolved.height,
            };
            state.grid.insert(index, resolved);
            size
        }
        _ => Size::ZERO,
    }
}

/// Yoga folds a node's own max-size into the measurement constraint before
/// calling the measure function; taffy only clamps a *definite* available
/// space, so an unconstrained probe still needs it applied here.
fn constrain(
    value: Option<f32>,
    max: Option<ir::Dim>,
    inset: f32,
    content_box: bool,
) -> Option<f32> {
    let max = match max {
        Some(ir::Dim::Px(v)) => {
            if content_box {
                v
            } else {
                (v - inset).max(0.0)
            }
        }
        _ => return value,
    };
    Some(match value {
        Some(v) => v.min(max),
        None => max,
    })
}

/// taffy hands the measure function an *inner* available space — already
/// reduced by padding, border and margin and clamped by min/max — while
/// `known_dimensions` is the outer size. Wrapping uses the inner space.
fn inner(avail: AvailableSpace) -> Option<f32> {
    match avail {
        AvailableSpace::Definite(v) => Some(v),
        _ => None,
    }
}

fn measure_text(
    flat: &Flat<'_>,
    index: usize,
    engine: &dyn TextEngine,
    state: &mut LayoutState,
    known: Size<Option<f32>>,
    avail: Size<AvailableSpace>,
    inset: Size<f32>,
) -> Size<f32> {
    let node = flat.nodes[index];
    let content = node.text().expect("text content");
    let content_box = content.block.content_box;

    let width = constrain(
        inner(avail.width),
        node.props.max_width,
        inset.width,
        content_box,
    );
    let height = constrain(
        inner(avail.height),
        node.props.max_height,
        inset.height,
        content_box,
    );
    let rotated = matches!(content.block.orientation, 90 | 270);

    // A min-content probe asks how narrow the text can get; wrapping at zero
    // yields the widest unbreakable piece.
    let min_content = matches!(
        if rotated { avail.height } else { avail.width },
        AvailableSpace::MinContent
    );

    // Rotated text wraps against its own height. Only a real constraint counts
    // — an available cross-axis size is the container's tentative guess, and
    // Yoga does not feed that back into the measure function.
    let rotated_limit = match node.props.height {
        Some(ir::Dim::Px(v)) => Some(if content_box {
            v
        } else {
            (v - inset.height).max(0.0)
        }),
        _ => match node.props.max_height {
            Some(ir::Dim::Px(v)) => Some(if content_box {
                v
            } else {
                (v - inset.height).max(0.0)
            }),
            _ => None,
        },
    };

    let wrap_width = if min_content {
        0.0
    } else if rotated {
        rotated_limit.unwrap_or(f32::INFINITY)
    } else {
        width.unwrap_or(f32::INFINITY)
    };

    let measure_at = |size: Option<f32>| {
        let paragraphs = create_paragraphs(content, wrap_width, engine, size);
        let w = paragraphs.iter().fold(0.0f32, |m, p| m.max(p.width));
        let h = paragraphs.iter().map(|p| p.height).sum::<f32>();
        (paragraphs, w, h)
    };

    let (mut paragraphs, mut block_w, mut block_h) = measure_at(None);
    let mut size = content.base.size;

    // Autofit binary-searches the largest size that still fits the constraint.
    if content.block.autofit && !content.block.nowrap {
        if let Some(limit) = height {
            size = fit(&measure_at, size, |_, h| h <= limit);
            let r = measure_at(Some(size));
            paragraphs = r.0;
            block_w = r.1;
            block_h = r.2;
        }
    } else if content.block.autofit && content.block.nowrap {
        if let Some(limit) = width {
            size = fit(&measure_at, size, |w, _| w <= limit);
            let r = measure_at(Some(size));
            paragraphs = r.0;
            block_w = r.1;
            block_h = r.2;
        }
    }

    if let Some(w) = width {
        block_w = block_w.min(w);
    }

    let _ = known;
    state.text.insert(index, TextLayout { paragraphs, size });

    if rotated {
        Size {
            width: block_h,
            height: block_w,
        }
    } else {
        Size {
            width: block_w,
            height: block_h,
        }
    }
}

/// Binary search over integer font sizes in 1..200, as the TS autofit does.
fn fit<F, P>(measure_at: &F, current: f32, fits: P) -> f32
where
    F: Fn(Option<f32>) -> (Vec<Paragraph>, f32, f32),
    P: Fn(f32, f32) -> bool,
{
    let mut min_size = 1.0f32;
    let mut max_size = 200.0f32;
    let mut optimal = if current > 0.0 { current } else { 12.0 };

    let m = measure_at(Some(optimal));
    if !fits(m.1, m.2) {
        max_size = optimal;
    }
    while max_size - min_size > 1.0 {
        let mid = ((min_size + max_size) / 2.0).floor();
        let m = measure_at(Some(mid));
        if fits(m.1, m.2) {
            min_size = mid;
            optimal = mid;
        } else {
            max_size = mid;
        }
    }
    optimal
}

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.0001
}

/// Yoga's `roundValueToPixelGrid` at a point scale factor of 1. The branches
/// are kept in Yoga's order even where two produce the same value.
#[allow(clippy::if_same_then_else)]
fn round_value(value: f32, force_ceil: bool, force_floor: bool) -> f32 {
    let fract = value % 1.0;
    if approx_eq(fract, 0.0) {
        value - fract
    } else if approx_eq(fract, 1.0) {
        value - fract + 1.0
    } else if force_ceil {
        value - fract + 1.0
    } else if force_floor {
        value - fract
    } else if fract > 0.5 || approx_eq(fract, 0.5) {
        value - fract + 1.0
    } else {
        value - fract
    }
}

fn has_fraction(v: f32) -> bool {
    let f = v % 1.0;
    !approx_eq(f, 0.0) && !approx_eq(f, 1.0)
}

fn collect(
    tree: &TaffyTree<usize>,
    flat: &Flat<'_>,
    id: NodeId,
    index: usize,
    state: &LayoutState,
    abs_x: f32,
    abs_y: f32,
) -> BoxLayout {
    let l = tree.layout(id).unwrap();
    let node = flat.nodes[index];

    // Nodes with a measure function round outward, so measured text is never
    // clipped by a fractional pixel. This mirrors Yoga's `textRounding`.
    let measured = matches!(node.content, Content::Text(_) | Content::Grid);

    let abs_left = abs_x + l.location.x;
    let abs_top = abs_y + l.location.y;

    let x = round_value(l.location.x, false, measured);
    let y = round_value(l.location.y, false, measured);
    let width = round_value(
        abs_left + l.size.width,
        measured && has_fraction(l.size.width),
        measured && !has_fraction(l.size.width),
    ) - round_value(abs_left, false, measured);
    let height = round_value(
        abs_top + l.size.height,
        measured && has_fraction(l.size.height),
        measured && !has_fraction(l.size.height),
    ) - round_value(abs_top, false, measured);

    let mut children = Vec::new();
    if let Some(grid) = state.grid.get(&index) {
        for child in &grid.children {
            let mut b = child.layout.clone();
            b.x = child.x;
            b.y = child.y;
            children.push(b);
        }
    } else {
        for (slot, &child_index) in flat.children[index].iter().enumerate() {
            let child_id = tree.child_at_index(id, slot).unwrap();
            children.push(collect(
                tree,
                flat,
                child_id,
                child_index,
                state,
                abs_left,
                abs_top,
            ));
        }
    }

    BoxLayout {
        index,
        id: node.id,
        ty: Some(node.ty),
        x,
        y,
        width,
        height,
        border: sides(l.border),
        padding: sides(l.padding),
        margin: sides(l.margin),
        children,
    }
}

/// Lay out a compiled tree. `width`/`height` constrain the root.
pub fn layout(
    root: &CompiledNode,
    engine: &dyn TextEngine,
    width: Option<f32>,
    height: Option<f32>,
) -> (BoxLayout, LayoutState) {
    let flat = Flat::new(root);
    let mut state = LayoutState::default();
    let out = layout_subtree(&flat, 0, engine, &mut state, width, height, None, None);
    (out, state)
}

/// Convenience for tests and `dump-layout`: sizes only, no measurement roots.
pub fn measure_child(
    flat: &Flat<'_>,
    index: usize,
    engine: &dyn TextEngine,
    state: &mut LayoutState,
    avail_w: Option<f32>,
    avail_h: Option<f32>,
    stretch_w: bool,
    stretch_h: bool,
) -> BoxLayout {
    let node = flat.nodes[index];
    let force_w = if stretch_w && node.props.width.is_none() {
        avail_w
    } else {
        None
    };
    let force_h = if stretch_h && node.props.height.is_none() {
        avail_h
    } else {
        None
    };
    layout_subtree(
        flat, index, engine, state, avail_w, avail_h, force_w, force_h,
    )
}

pub use ir::NodeType as _NodeType;
