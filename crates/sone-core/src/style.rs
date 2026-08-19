use kurbo::BezPath;

use crate::css::color::{parse_color, Color, BLACK};
use crate::css::filter::FilterOp;
use crate::css::gradient::Gradient;
use crate::css::shadow::CssShadow;
use crate::ir;
use crate::paint::{DropShadowSpec, ImageHandle, SpanStyle};
use crate::text::bidi::{BaseDir, Dir};

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Color(Color),
    Gradient(Vec<Gradient>),
}

impl Paint {
    pub fn solid_or_black(&self) -> Color {
        match self {
            Paint::Color(c) => *c,
            Paint::Gradient(_) => BLACK,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BgLayer {
    Color(Color),
    Gradient(Gradient),
    Photo(Box<CompiledNode>),
}

/// Text style carried by every run — spans inherit these from their block.
#[derive(Debug, Clone, PartialEq)]
pub struct RunStyle {
    pub tag: Option<String>,
    pub size: f32,
    pub color: Paint,
    pub font: Vec<String>,
    pub italic: bool,
    pub oblique: bool,
    pub weight: i32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub stroke_color: Option<Color>,
    pub stroke_width: f32,
    pub underline: f32,
    pub underline_color: Option<Color>,
    pub overline: f32,
    pub overline_color: Option<Color>,
    pub line_through: f32,
    pub line_through_color: Option<Color>,
    pub highlight_color: Option<Color>,
    pub drop_shadows: Vec<CssShadow>,
    pub offset_y: f32,
    pub text_dir: Option<Dir>,
}

impl Default for RunStyle {
    fn default() -> Self {
        RunStyle {
            tag: None,
            size: 11.0,
            color: Paint::Color(BLACK),
            font: vec!["sans-serif".into()],
            italic: false,
            oblique: false,
            weight: 400,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            stroke_color: Some(BLACK),
            stroke_width: 0.0,
            underline: 0.0,
            underline_color: None,
            overline: 0.0,
            overline_color: None,
            line_through: 0.0,
            line_through_color: None,
            highlight_color: None,
            drop_shadows: Vec::new(),
            offset_y: 0.0,
            text_dir: None,
        }
    }
}

impl RunStyle {
    pub fn shaping(&self) -> SpanStyle {
        SpanStyle {
            font: self.font.clone(),
            size: self.size,
            weight: self.weight,
            italic: self.italic,
            oblique: self.oblique,
            letter_spacing: self.letter_spacing,
            word_spacing: self.word_spacing,
        }
    }

    /// Apply the props a span inherits from its text block, then its own.
    pub fn apply(&mut self, p: &ir::Props) {
        if let Some(v) = p.tag.clone() {
            self.tag = Some(v)
        }
        if let Some(v) = p.size {
            self.size = v
        }
        if let Some(v) = &p.color {
            self.color = parse_paint(v)
        }
        if let Some(v) = &p.font {
            self.font = v.clone()
        }
        if let Some(v) = p.style {
            self.italic = v == ir::FontStyle::Italic;
            self.oblique = v == ir::FontStyle::Oblique;
        }
        if let Some(v) = &p.weight {
            self.weight = v.resolve()
        }
        if let Some(v) = p.letter_spacing {
            self.letter_spacing = v
        }
        if let Some(v) = p.word_spacing {
            self.word_spacing = v
        }
        if let Some(v) = &p.stroke_color {
            self.stroke_color = Some(parse_color(v))
        }
        if let Some(v) = p.stroke_width {
            self.stroke_width = v
        }
        if let Some(v) = p.underline {
            self.underline = v
        }
        if let Some(v) = &p.underline_color {
            self.underline_color = v.as_deref().map(parse_color)
        }
        if let Some(v) = p.overline {
            self.overline = v
        }
        if let Some(v) = &p.overline_color {
            self.overline_color = v.as_deref().map(parse_color)
        }
        if let Some(v) = p.line_through {
            self.line_through = v
        }
        if let Some(v) = &p.line_through_color {
            self.line_through_color = v.as_deref().map(parse_color)
        }
        if let Some(v) = &p.highlight_color {
            self.highlight_color = v.as_deref().map(parse_color)
        }
        if let Some(v) = &p.drop_shadows {
            self.drop_shadows = resolve_shadow_list(v)
        }
        if let Some(v) = p.offset_y {
            self.offset_y = v
        }
        if let Some(v) = p.text_dir {
            self.text_dir = Some(if v == ir::Direction::Rtl {
                Dir::Rtl
            } else {
                Dir::Ltr
            });
        }
    }
}

/// Text-block properties that spans do not inherit.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStyle {
    pub nowrap: bool,
    pub max_lines: Option<usize>,
    pub line_break: ir::LineBreakMode,
    pub text_overflow: ir::TextOverflow,
    pub line_height: Option<f32>,
    pub indent_size: f32,
    pub hanging_indent_size: f32,
    pub align: Option<ir::TextAlign>,
    pub text_wrap: Option<ir::TextWrap>,
    pub autofit: bool,
    pub base_dir: Option<BaseDir>,
    pub orientation: u32,
    pub tab_stops: Vec<f32>,
    pub tab_leader: String,
    pub content_box: bool,
}

impl Default for BlockStyle {
    fn default() -> Self {
        BlockStyle {
            nowrap: false,
            max_lines: None,
            line_break: ir::LineBreakMode::Greedy,
            text_overflow: ir::TextOverflow::Clip,
            line_height: None,
            indent_size: 0.0,
            hanging_indent_size: 0.0,
            align: Some(ir::TextAlign::Left),
            text_wrap: None,
            autofit: false,
            base_dir: None,
            orientation: 0,
            tab_stops: Vec::new(),
            tab_leader: String::new(),
            content_box: true,
        }
    }
}

impl BlockStyle {
    pub fn apply(&mut self, p: &ir::Props) {
        if let Some(v) = p.nowrap {
            self.nowrap = v
        }
        if let Some(v) = p.max_lines {
            self.max_lines = if v.is_finite() {
                Some(v.max(0.0).floor() as usize)
            } else {
                None
            };
        }
        if let Some(v) = p.line_break {
            self.line_break = v
        }
        if let Some(v) = p.text_overflow {
            self.text_overflow = v
        }
        if let Some(v) = p.line_height {
            self.line_height = if v.is_finite() { Some(v) } else { None };
        }
        if let Some(v) = p.indent_size {
            self.indent_size = v
        }
        if let Some(v) = p.hanging_indent_size {
            self.hanging_indent_size = v
        }
        if let Some(v) = p.align {
            self.align = Some(v)
        }
        if let Some(v) = p.text_wrap {
            self.text_wrap = Some(v)
        }
        if let Some(v) = p.autofit {
            self.autofit = v
        }
        if let Some(v) = p.base_dir {
            self.base_dir = Some(v)
        }
        if let Some(v) = p.orientation {
            self.orientation = v
        }
        if let Some(v) = &p.tab_stops {
            self.tab_stops = v.clone()
        }
        if let Some(v) = &p.tab_leader {
            self.tab_leader = v.clone()
        }
        if let Some(v) = p.box_sizing {
            self.content_box = v == ir::BoxSizing::ContentBox
        }
    }
}

#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Span { text: String, style: RunStyle },
}

impl Inline {
    pub fn text(&self) -> &str {
        match self {
            Inline::Text(t) => t,
            Inline::Span { text, .. } => text,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextContent {
    pub base: RunStyle,
    pub block: BlockStyle,
    pub inlines: Vec<Inline>,
    pub clip_image: Option<Box<CompiledNode>>,
}

impl TextContent {
    /// Style for one inline: spans carry their own, plain text uses the block's.
    pub fn style_for<'a>(&'a self, inline: &'a Inline) -> &'a RunStyle {
        match inline {
            Inline::Text(_) => &self.base,
            Inline::Span { style, .. } => style,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhotoContent {
    pub image: Option<ImageHandle>,
    pub scale_type: ir::ScaleType,
    pub scale_alignment: f32,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub clip_path: Option<BezPath>,
}

#[derive(Debug, Clone)]
pub struct PathContent {
    pub d: String,
    pub path: BezPath,
    /// [left, top, right, bottom] of the unscaled path.
    pub bounds: [f64; 4],
    pub stroke: Option<Color>,
    pub stroke_spec: crate::paint::StrokeSpec,
    pub fill: Option<Paint>,
    pub fill_opacity: f32,
    pub even_odd: bool,
    pub scale_path: f64,
}

#[derive(Debug, Clone, Default)]
pub struct BoxStyle {
    pub background: Vec<BgLayer>,
    pub border_color: Color,
    pub corner_radius: Vec<f64>,
    pub corner_smoothing: Option<f64>,
    pub cut_corners: bool,
    pub opacity: f32,
    pub shadows: Vec<DropShadowSpec>,
    pub filters: Vec<FilterOp>,
    pub rotation: f32,
    pub scale: Option<[f32; 2]>,
    pub translate_x: f32,
    pub translate_y: f32,
}

#[derive(Debug, Clone)]
pub enum Content {
    Container,
    Grid,
    Text(Box<TextContent>),
    Photo(Box<PhotoContent>),
    Path(Box<PathContent>),
    Table,
    List,
    ClipGroup(Option<BezPath>),
}

#[derive(Debug, Clone)]
pub struct CompiledNode {
    pub id: u32,
    pub ty: ir::NodeType,
    /// Layout props, after compile-time defaults are applied.
    pub props: ir::Props,
    pub boxed: BoxStyle,
    pub content: Content,
    pub children: Vec<CompiledNode>,
}

impl CompiledNode {
    pub fn text(&self) -> Option<&TextContent> {
        match &self.content {
            Content::Text(t) => Some(t),
            _ => None,
        }
    }
    pub fn text_mut(&mut self) -> Option<&mut TextContent> {
        match &mut self.content {
            Content::Text(t) => Some(t),
            _ => None,
        }
    }
}

pub fn parse_paint(value: &str) -> Paint {
    use crate::css::gradient::{is_color, parse_gradients};
    if is_color(value) {
        return Paint::Color(parse_color(value));
    }
    match parse_gradients(value) {
        Ok(g) if !g.is_empty() => Paint::Gradient(g),
        _ => Paint::Color(parse_color(value)),
    }
}

pub fn resolve_shadow_list(list: &[String]) -> Vec<CssShadow> {
    list.iter()
        .flat_map(|s| crate::css::shadow::parse_shadow(s))
        .collect()
}

/// A parsed CSS shadow becomes a Skia-ready spec: sigma is blur/2.
pub fn shadow_spec(shadow: &CssShadow, default_color: Color) -> DropShadowSpec {
    DropShadowSpec {
        dx: shadow.offset_x as f32,
        dy: shadow.offset_y as f32,
        sigma: (shadow.blur_radius / 2.0) as f32,
        color: shadow
            .color
            .as_deref()
            .map(parse_color)
            .unwrap_or(default_color),
        spread: shadow.spread_radius.unwrap_or(0.0) as f32,
        inset: shadow.inset,
    }
}
