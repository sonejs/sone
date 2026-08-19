use kurbo::{BezPath, Shape};

use crate::css::color::{parse_color, BLACK};
use crate::css::filter::parse_css_filter;
use crate::css::gradient::parse_gradients;
use crate::error::SoneError;
use crate::ir::{self, Node, NodeType, Props};
use crate::paint::{ImageHandle, StrokeSpec};
use crate::style::*;
use crate::Result;

pub trait AssetLoader {
    fn load_image(&self, src: &str) -> Result<ImageHandle>;
}

/// Layout-only loader: every image resolves to a 1x1 placeholder.
pub struct NullAssets;

impl AssetLoader for NullAssets {
    fn load_image(&self, _src: &str) -> Result<ImageHandle> {
        Ok(ImageHandle {
            width: 1,
            height: 1,
            inner: std::sync::Arc::new(()),
        })
    }
}

pub struct CompileCtx<'a> {
    pub assets: &'a dyn AssetLoader,
    pub strict: bool,
    next_id: u32,
    warnings: Vec<String>,
}

impl<'a> CompileCtx<'a> {
    pub fn new(assets: &'a dyn AssetLoader) -> Self {
        CompileCtx {
            assets,
            strict: false,
            next_id: 0,
            warnings: Vec::new(),
        }
    }
    fn id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
    fn warn(&mut self, message: String) {
        if !self.warnings.contains(&message) {
            self.warnings.push(message);
        }
    }
}

#[derive(Clone, Default)]
struct TextDefaults {
    run: RunStyle,
    block: BlockStyle,
}

pub fn compile(root: &Node, ctx: &mut CompileCtx<'_>) -> Result<Option<CompiledNode>> {
    let defaults = TextDefaults::default();
    compile_node(root, &defaults, ctx)
}

fn parse_path_d(d: &str) -> BezPath {
    BezPath::from_svg(d).unwrap_or_default()
}

fn box_style(props: &Props, ctx: &mut CompileCtx<'_>) -> Result<BoxStyle> {
    let mut out = BoxStyle {
        border_color: props
            .border_color
            .as_deref()
            .map(parse_color)
            .unwrap_or(BLACK),
        corner_radius: props
            .corner_radius
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|v| *v as f64)
            .collect(),
        corner_smoothing: props.corner_smoothing.map(|v| v as f64),
        cut_corners: props.corner == Some(ir::Corner::Cut),
        opacity: props.opacity.unwrap_or(1.0),
        rotation: props.rotation.unwrap_or(0.0),
        scale: props.scale,
        translate_x: props.translate_x.unwrap_or(0.0),
        translate_y: props.translate_y.unwrap_or(0.0),
        ..Default::default()
    };

    if let Some(shadows) = &props.shadows {
        out.shadows = resolve_shadow_list(shadows)
            .iter()
            .map(|s| shadow_spec(s, BLACK))
            .collect();
    }
    if let Some(filters) = &props.filters {
        let (ops, unknown) = parse_css_filter(filters);
        for name in unknown {
            ctx.warn(format!(
                "CSS filter {name:?} is not supported and was ignored"
            ));
        }
        out.filters = ops;
    }
    Ok(out)
}

/// `compile`'s background pass: colour strings stay colours, gradient strings
/// expand to one layer per gradient, photo layers load their image.
fn background_layers(
    props: &Props,
    defaults: &TextDefaults,
    ctx: &mut CompileCtx<'_>,
) -> Result<Vec<BgLayer>> {
    let Some(list) = &props.background else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for bg in list {
        match bg {
            ir::Background::Css(s) => {
                if crate::css::gradient::is_color(s) {
                    out.push(BgLayer::Color(parse_color(s)));
                } else {
                    match parse_gradients(s) {
                        Ok(gradients) if !gradients.is_empty() => {
                            out.extend(gradients.into_iter().map(BgLayer::Gradient));
                        }
                        _ => out.push(BgLayer::Color(parse_color(s))),
                    }
                }
            }
            ir::Background::Photo(node) => {
                if let Some(compiled) = compile_node(node, defaults, ctx)? {
                    out.push(BgLayer::Photo(Box::new(compiled)));
                }
            }
        }
    }
    Ok(out)
}

fn configure_photo(props: &mut Props, image: &ImageHandle) {
    let ratio = image.width as f32 / image.height.max(1) as f32;
    if props.preserve_aspect_ratio == Some(true) {
        match (props.width, props.height) {
            (Some(ir::Dim::Px(w)), None) => props.height = Some(ir::Dim::Px((w / ratio).round())),
            (None, Some(ir::Dim::Px(h))) => props.width = Some(ir::Dim::Px((h * ratio).round())),
            _ => {}
        }
    }
    if props.width.is_none() {
        props.width = Some(ir::Dim::Px(image.width as f32));
    }
    if props.height.is_none() {
        props.height = Some(ir::Dim::Px(image.height as f32));
    }
}

fn compile_photo(
    node: &Node,
    defaults: &TextDefaults,
    ctx: &mut CompileCtx<'_>,
) -> Result<Option<CompiledNode>> {
    let id = ctx.id();
    let mut props = node.props.clone();
    let boxed = box_style(&props, ctx)?;
    let background = background_layers(&props, defaults, ctx)?;

    let Some(src) = props.src.clone() else {
        return Ok(None);
    };
    let image = ctx.assets.load_image(&src)?;
    configure_photo(&mut props, &image);

    let content = PhotoContent {
        image: Some(image),
        scale_type: props.scale_type.unwrap_or(ir::ScaleType::Fill),
        scale_alignment: props.scale_alignment.unwrap_or(0.5),
        flip_horizontal: props.flip_horizontal.unwrap_or(false),
        flip_vertical: props.flip_vertical.unwrap_or(false),
        clip_path: props.clip_path.as_deref().map(parse_path_d),
    };

    Ok(Some(CompiledNode {
        id,
        ty: NodeType::Photo,
        props,
        boxed: BoxStyle {
            background,
            ..boxed
        },
        content: Content::Photo(Box::new(content)),
        children: Vec::new(),
    }))
}

fn compile_text(
    node: &Node,
    defaults: &TextDefaults,
    ctx: &mut CompileCtx<'_>,
) -> Result<Option<CompiledNode>> {
    let id = ctx.id();
    let mut props = node.props.clone();
    if props.flex_shrink.is_none() {
        props.flex_shrink = Some(1.0);
    }
    if props.box_sizing.is_none() {
        props.box_sizing = Some(ir::BoxSizing::ContentBox);
    }

    let mut base = defaults.run.clone();
    base.apply(&props);
    let mut block = defaults.block.clone();
    block.apply(&props);

    let mut inlines = Vec::new();
    for item in &node.inline {
        match item {
            ir::Inline::Text(t) => inlines.push(Inline::Text(t.clone())),
            ir::Inline::Span(span) => {
                let mut style = base.clone();
                // Only the inheritable subset carries over; block-level props do not.
                style.offset_y = 0.0;
                style.text_dir = None;
                style.tag = None;
                style.apply(&span.props);
                let text = span
                    .inline
                    .iter()
                    .map(|i| match i {
                        ir::Inline::Text(t) => t.clone(),
                        ir::Inline::Span(s) => s.inline.iter().map(inline_text).collect(),
                    })
                    .collect::<String>();
                inlines.push(Inline::Span { text, style });
            }
        }
    }

    let clip_image = match &props.clip_image {
        Some(n) => compile_node(n, defaults, ctx)?.map(Box::new),
        None => None,
    };

    let boxed = box_style(&props, ctx)?;
    let background = background_layers(&props, defaults, ctx)?;

    Ok(Some(CompiledNode {
        id,
        ty: NodeType::Text,
        props,
        boxed: BoxStyle {
            background,
            ..boxed
        },
        content: Content::Text(Box::new(TextContent {
            base,
            block,
            inlines,
            clip_image,
        })),
        children: Vec::new(),
    }))
}

fn inline_text(i: &ir::Inline) -> String {
    match i {
        ir::Inline::Text(t) => t.clone(),
        ir::Inline::Span(s) => s.inline.iter().map(inline_text).collect(),
    }
}

fn compile_path(node: &Node, ctx: &mut CompileCtx<'_>) -> Result<Option<CompiledNode>> {
    let id = ctx.id();
    let props = node.props.clone();
    let d = props.d.clone().unwrap_or_default();
    let path = parse_path_d(&d);
    let bb = path.bounding_box();
    let bounds = [bb.x0, bb.y0, bb.x1, bb.y1];

    let stroke_spec = StrokeSpec {
        width: props.stroke_width.unwrap_or(1.0),
        cap: match props.stroke_line_cap {
            Some(ir::StrokeCap::Round) => crate::paint::Cap::Round,
            Some(ir::StrokeCap::Square) => crate::paint::Cap::Square,
            _ => crate::paint::Cap::Butt,
        },
        join: match props.stroke_line_join {
            Some(ir::StrokeJoin::Round) => crate::paint::Join::Round,
            Some(ir::StrokeJoin::Bevel) => crate::paint::Join::Bevel,
            _ => crate::paint::Join::Miter,
        },
        miter_limit: props.stroke_miter_limit.unwrap_or(4.0),
        // Canvas2D duplicates an odd-length dash array; Skia requires an even one.
        dash: props
            .stroke_dash_array
            .clone()
            .filter(|d| !d.is_empty())
            .map(|d| {
                if d.len() % 2 == 1 {
                    d.iter().chain(d.iter()).copied().collect()
                } else {
                    d
                }
            }),
        dash_offset: props.stroke_dash_offset.unwrap_or(0.0),
    };

    let content = PathContent {
        d,
        path,
        bounds,
        stroke: props.stroke.as_deref().map(parse_color),
        stroke_spec,
        fill: props.fill.as_deref().map(parse_paint),
        fill_opacity: props.fill_opacity.unwrap_or(1.0),
        even_odd: props.fill_rule == Some(ir::FillRule::EvenOdd),
        scale_path: props.scale_path.unwrap_or(1.0) as f64,
    };

    let boxed = box_style(&props, ctx)?;
    Ok(Some(CompiledNode {
        id,
        ty: NodeType::Path,
        props,
        boxed,
        content: Content::Path(Box::new(content)),
        children: Vec::new(),
    }))
}

fn marker_text(list_style: Option<&ir::ListStyle>, index: usize, start_index: i32) -> String {
    let name = match list_style {
        Some(ir::ListStyle::Name(n)) => n.as_str(),
        _ => "disc",
    };
    match name {
        "disc" => "•".into(),
        "circle" => "◦".into(),
        "square" => "▪".into(),
        "decimal" => format!("{}.", start_index + index as i32),
        "dash" => "–".into(),
        "none" => String::new(),
        other => other.to_string(),
    }
}

/// Rebuild each list item as `[marker, content]`, as `compile` does in TS.
fn build_list_items(node: &Node) -> Vec<Node> {
    let props = &node.props;
    let start_index = props.start_index.unwrap_or(1);
    let marker_gap = props.marker_gap.unwrap_or(8.0);
    let marker_offset = props.marker_offset.unwrap_or(0.0);

    let mut out = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        let mut item = child.clone();

        let mut marker = match (&item.props.marker, &props.list_style) {
            (Some(m), _) => (**m).clone(),
            (None, Some(ir::ListStyle::Span(span))) => {
                let mut n = Node::new(NodeType::Text);
                let mut s = (**span).clone();
                let joined: String = s.inline.iter().map(inline_text).collect();
                if joined.contains("{}") {
                    s.inline = vec![ir::Inline::Text(
                        joined.replace("{}", &format!("{}", start_index + index as i32)),
                    )];
                }
                n.props = s.props.clone();
                n.inline = s.inline.clone();
                n
            }
            _ => {
                let mut n = Node::new(NodeType::Text);
                n.inline = vec![ir::Inline::Text(marker_text(
                    props.list_style.as_ref(),
                    index,
                    start_index,
                ))];
                n
            }
        };
        if marker.ty() == NodeType::Span {
            let mut n = Node::new(NodeType::Text);
            n.props = marker.props.clone();
            n.inline = marker.inline.clone();
            marker = n;
        }
        marker.props.marker = None;
        marker.props.nowrap = Some(true);
        if marker_offset != 0.0 {
            marker.props.margin_top = Some(ir::Dim::Px(marker_offset));
            marker.props.margin_bottom = Some(ir::Dim::Px(marker_offset));
        }
        // Align the marker baseline with the first line of item content.
        if let Some(lh) = item
            .children
            .iter()
            .find(|c| {
                c.ty() == NodeType::Text && c.props.line_height.is_some_and(|v| v.is_finite())
            })
            .and_then(|c| c.props.line_height)
        {
            marker.props.line_height = Some(lh);
        }

        let mut content = Node::new(NodeType::Column);
        content.props.flex = Some(1.0);
        content.children = std::mem::take(&mut item.children);

        item.props.marker = None;
        item.children = vec![marker, content];
        if item.props.flex_direction.is_none() {
            item.props.flex_direction = Some(ir::FlexDirection::Row);
        }
        if item.props.gap.is_none() {
            item.props.gap = Some(marker_gap);
        }
        if item.props.align_items.is_none() {
            item.props.align_items = Some(ir::AlignItems::FlexStart);
        }
        out.push(item);
    }
    out
}

fn compile_node(
    node: &Node,
    defaults: &TextDefaults,
    ctx: &mut CompileCtx<'_>,
) -> Result<Option<CompiledNode>> {
    match node.ty() {
        NodeType::TextDefault => Err(SoneError::Layout(
            "text-default nodes are flattened during compilation".into(),
        )),
        NodeType::Text => compile_text(node, defaults, ctx),
        NodeType::Photo => compile_photo(node, defaults, ctx),
        NodeType::Path => compile_path(node, ctx),
        NodeType::Span => Err(SoneError::Layout(
            "span nodes may only appear inside text".into(),
        )),
        _ => compile_container(node, defaults, ctx),
    }
}

fn compile_container(
    node: &Node,
    defaults: &TextDefaults,
    ctx: &mut CompileCtx<'_>,
) -> Result<Option<CompiledNode>> {
    let id = ctx.id();
    let ty = node.ty();
    let mut props = node.props.clone();

    match ty {
        NodeType::Row | NodeType::TableRow => {
            if props.flex_direction.is_none() {
                props.flex_direction = Some(ir::FlexDirection::Row);
            }
        }
        NodeType::List if props.flex_direction.is_none() => {
            props.flex_direction = Some(ir::FlexDirection::Column);
        }
        _ => {}
    }

    let boxed = box_style(&props, ctx)?;
    let background = background_layers(&props, defaults, ctx)?;

    let source_children: Vec<Node> = if ty == NodeType::List {
        build_list_items(node)
    } else {
        node.children.clone()
    };

    let mut children = Vec::new();
    for child in &source_children {
        if child.ty() == NodeType::TextDefault {
            // A text-default node contributes no box; it only cascades style.
            let mut nested = defaults.clone();
            nested.run.apply(&child.props);
            nested.block.apply(&child.props);
            for c in &child.children {
                if let Some(compiled) = compile_node(c, &nested, ctx)? {
                    children.push(compiled);
                }
            }
            continue;
        }
        if let Some(compiled) = compile_node(child, defaults, ctx)? {
            children.push(compiled);
        }
    }

    if ty == NodeType::Table {
        apply_table_spacing(&props, &mut children);
    }

    let content = match ty {
        NodeType::Grid => Content::Grid,
        NodeType::Table => Content::Table,
        NodeType::List => Content::List,
        NodeType::ClipGroup => Content::ClipGroup(props.clip_path.as_deref().map(parse_path_d)),
        _ => Content::Container,
    };

    Ok(Some(CompiledNode {
        id,
        ty,
        props,
        boxed: BoxStyle {
            background,
            ..boxed
        },
        content,
        children,
    }))
}

fn apply_table_spacing(props: &Props, rows: &mut [CompiledNode]) {
    let Some(spacing) = &props.spacing else {
        return;
    };
    let Some(&x) = spacing.first() else { return };
    let y = spacing.get(1).copied().unwrap_or(x);
    for row in rows.iter_mut() {
        for cell in row.children.iter_mut() {
            let p = &mut cell.props;
            if p.padding_left.is_none() {
                p.padding_left = Some(ir::Dim::Px(x))
            }
            if p.padding_right.is_none() {
                p.padding_right = Some(ir::Dim::Px(x))
            }
            if p.padding_top.is_none() {
                p.padding_top = Some(ir::Dim::Px(y))
            }
            if p.padding_bottom.is_none() {
                p.padding_bottom = Some(ir::Dim::Px(y))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Document;

    fn compile_json(json: &str) -> CompiledNode {
        let doc = Document::from_json(json).unwrap();
        let assets = NullAssets;
        let mut ctx = CompileCtx::new(&assets);
        compile(&doc.root, &mut ctx).unwrap().unwrap()
    }

    #[test]
    fn row_defaults_to_row_direction() {
        let n = compile_json(r#"{"sone":1,"root":{"type":"row"}}"#);
        assert_eq!(n.props.flex_direction, Some(ir::FlexDirection::Row));
    }

    #[test]
    fn text_gets_shrink_and_content_box() {
        let n = compile_json(r#"{"sone":1,"root":{"type":"text","inline":["hi"]}}"#);
        assert_eq!(n.props.flex_shrink, Some(1.0));
        assert_eq!(n.props.box_sizing, Some(ir::BoxSizing::ContentBox));
        assert_eq!(n.text().unwrap().base.size, 11.0);
    }

    #[test]
    fn spans_inherit_the_block_style() {
        let n = compile_json(
            r#"{"sone":1,"root":{"type":"text","props":{"size":20,"color":"red","weight":"bold"},
                 "inline":["a",{"type":"span","props":{"color":"blue"},"inline":["b"]}]}}"#,
        );
        let t = n.text().unwrap();
        match &t.inlines[1] {
            Inline::Span { style, .. } => {
                assert_eq!(style.size, 20.0);
                assert_eq!(style.weight, 700);
                assert_eq!(style.color, Paint::Color(parse_color("blue")));
            }
            _ => panic!("expected a span"),
        }
    }

    #[test]
    fn text_default_cascades_and_flattens() {
        let n = compile_json(
            r#"{"sone":1,"root":{"type":"column","children":[
                 {"type":"text-default","props":{"size":30},"children":[
                   {"type":"text","inline":["x"]}]}]}}"#,
        );
        assert_eq!(n.children.len(), 1);
        assert_eq!(n.children[0].ty, NodeType::Text);
        assert_eq!(n.children[0].text().unwrap().base.size, 30.0);
    }

    #[test]
    fn gradient_background_expands_per_gradient() {
        let n = compile_json(
            r##"{"sone":1,"root":{"type":"column","props":{"background":
                 ["linear-gradient(red, blue), linear-gradient(green, black)","#fff"]}}}"##,
        );
        assert_eq!(n.boxed.background.len(), 3);
        assert!(matches!(n.boxed.background[0], BgLayer::Gradient(_)));
        assert!(matches!(n.boxed.background[2], BgLayer::Color(_)));
    }

    #[test]
    fn list_items_are_rebuilt_with_markers() {
        let n = compile_json(
            r#"{"sone":1,"root":{"type":"list","props":{"listStyle":"decimal","startIndex":3},
                 "children":[{"type":"list-item","children":[{"type":"text","inline":["a"]}]}]}}"#,
        );
        let item = &n.children[0];
        assert_eq!(item.props.flex_direction, Some(ir::FlexDirection::Row));
        assert_eq!(item.props.gap, Some(8.0));
        assert_eq!(item.children.len(), 2);
        let marker = item.children[0].text().unwrap();
        assert!(matches!(&marker.inlines[0], Inline::Text(t) if t == "3."));
        assert_eq!(item.children[1].ty, NodeType::Column);
    }

    #[test]
    fn resolved_markers_win_over_list_style() {
        let n = compile_json(
            r#"{"sone":1,"root":{"type":"list","props":{"listStyle":"disc"},"children":[
                 {"type":"list-item","props":{"marker":{"type":"span","props":{"color":"red"},"inline":["→"]}},
                  "children":[{"type":"text","inline":["a"]}]}]}}"#,
        );
        let marker = n.children[0].children[0].text().unwrap();
        assert!(matches!(&marker.inlines[0], Inline::Text(t) if t == "→"));
        assert_eq!(marker.base.color, Paint::Color(parse_color("red")));
    }

    #[test]
    fn table_spacing_becomes_cell_padding() {
        let n = compile_json(
            r#"{"sone":1,"root":{"type":"table","props":{"spacing":[6,4]},"children":[
                 {"type":"table-row","children":[{"type":"table-cell"}]}]}}"#,
        );
        let cell = &n.children[0].children[0];
        assert_eq!(cell.props.padding_left, Some(ir::Dim::Px(6.0)));
        assert_eq!(cell.props.padding_top, Some(ir::Dim::Px(4.0)));
    }

    #[test]
    fn ids_are_assigned_in_pre_order() {
        let n = compile_json(
            r#"{"sone":1,"root":{"type":"column","children":[
                 {"type":"row","children":[{"type":"text","inline":["a"]}]},
                 {"type":"text","inline":["b"]}]}}"#,
        );
        assert_eq!(n.id, 0);
        assert_eq!(n.children[0].id, 1);
        assert_eq!(n.children[0].children[0].id, 2);
        assert_eq!(n.children[1].id, 3);
    }

    #[test]
    fn photo_takes_its_intrinsic_size() {
        let n = compile_json(r#"{"sone":1,"root":{"type":"photo","props":{"src":"x.png"}}}"#);
        assert_eq!(n.props.width, Some(ir::Dim::Px(1.0)));
        assert_eq!(n.props.height, Some(ir::Dim::Px(1.0)));
    }

    #[test]
    fn path_bounds_come_from_the_geometry() {
        let n =
            compile_json(r#"{"sone":1,"root":{"type":"path","props":{"d":"M10,10 L50,30 Z"}}}"#);
        match &n.content {
            Content::Path(p) => assert_eq!(p.bounds, [10.0, 10.0, 50.0, 30.0]),
            _ => panic!("expected a path"),
        }
    }
}
