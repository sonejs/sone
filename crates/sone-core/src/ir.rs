use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::text::bidi::BaseDir;

/// A length that may be `auto`, a percentage, or a pixel number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dim {
    Px(f32),
    Percent(f32),
    Auto,
}

impl Serialize for Dim {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Dim::Px(v) => s.serialize_f32(*v),
            Dim::Percent(v) => s.serialize_str(&format!("{v}%")),
            Dim::Auto => s.serialize_str("auto"),
        }
    }
}

impl<'de> Deserialize<'de> for Dim {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Num(f32),
            Str(String),
        }
        match Raw::deserialize(d)? {
            Raw::Num(v) => Ok(Dim::Px(v)),
            Raw::Str(s) => {
                let t = s.trim();
                if t == "auto" {
                    return Ok(Dim::Auto);
                }
                if let Some(p) = t.strip_suffix('%') {
                    return p.parse().map(Dim::Percent).map_err(|_| {
                        serde::de::Error::custom(format!("invalid percentage {s:?}"))
                    });
                }
                t.parse()
                    .map(Dim::Px)
                    .map_err(|_| serde::de::Error::custom(format!("invalid length {s:?}")))
            }
        }
    }
}

/// A grid track: a fixed pixel size, `auto`, or an `fr` share.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridTrack {
    Fixed(f32),
    Auto,
    Fr(f32),
}

impl Serialize for GridTrack {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            GridTrack::Fixed(v) => s.serialize_f32(*v),
            GridTrack::Auto => s.serialize_str("auto"),
            GridTrack::Fr(v) => s.serialize_str(&format!("{v}fr")),
        }
    }
}

impl<'de> Deserialize<'de> for GridTrack {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Num(f32),
            Str(String),
        }
        match Raw::deserialize(d)? {
            Raw::Num(v) => Ok(GridTrack::Fixed(v)),
            Raw::Str(s) => {
                if s == "auto" {
                    return Ok(GridTrack::Auto);
                }
                if let Some(p) = s.strip_suffix("fr") {
                    if let Ok(v) = p.parse() {
                        return Ok(GridTrack::Fr(v));
                    }
                }
                Err(serde::de::Error::custom(format!(
                    "Invalid grid track: {s:?}"
                )))
            }
        }
    }
}

macro_rules! kw_enum {
    ($name:ident { $($variant:ident => $text:literal),* $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $text)] $variant),*
        }
    };
}

kw_enum!(AlignContent {
    FlexStart => "flex-start", FlexEnd => "flex-end", Center => "center",
    Stretch => "stretch", SpaceBetween => "space-between",
    SpaceAround => "space-around", SpaceEvenly => "space-evenly",
});
kw_enum!(AlignItems {
    FlexStart => "flex-start", FlexEnd => "flex-end", Center => "center",
    Stretch => "stretch", Baseline => "baseline",
});
kw_enum!(JustifyContent {
    FlexStart => "flex-start", FlexEnd => "flex-end", Center => "center",
    SpaceBetween => "space-between", SpaceAround => "space-around",
    SpaceEvenly => "space-evenly",
});
kw_enum!(FlexDirection {
    Row => "row", Column => "column", RowReverse => "row-reverse",
    ColumnReverse => "column-reverse",
});
kw_enum!(FlexWrap { Wrap => "wrap", NoWrap => "nowrap", WrapReverse => "wrap-reverse" });
kw_enum!(BoxSizing { BorderBox => "border-box", ContentBox => "content-box" });
kw_enum!(Direction { Ltr => "ltr", Rtl => "rtl" });
kw_enum!(Display { None => "none", Flex => "flex", Contents => "contents" });
kw_enum!(Overflow { Visible => "visible", Hidden => "hidden", Scroll => "scroll" });
kw_enum!(Position { Absolute => "absolute", Relative => "relative", Static => "static" });
kw_enum!(PageBreak { Before => "before", After => "after", Avoid => "avoid" });
kw_enum!(Corner { Cut => "cut", Round => "round" });
kw_enum!(ScaleType { Cover => "cover", Fill => "fill", Contain => "contain" });
kw_enum!(FontStyle { Normal => "normal", Italic => "italic", Oblique => "oblique" });
kw_enum!(LineBreakMode { Greedy => "greedy", KnuthPlass => "knuth-plass" });
kw_enum!(TextOverflow { Clip => "clip", Ellipsis => "ellipsis" });
kw_enum!(TextAlign { Left => "left", Right => "right", Center => "center", Justify => "justify" });
kw_enum!(TextWrap { Wrap => "wrap", Balance => "balance" });
kw_enum!(StrokeCap { Butt => "butt", Round => "round", Square => "square" });
kw_enum!(StrokeJoin { Bevel => "bevel", Miter => "miter", Round => "round" });
kw_enum!(FillRule { EvenOdd => "evenodd", NonZero => "nonzero" });

/// `weight` accepts CSS keywords or a number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Weight {
    Num(f32),
    Kw(String),
}

impl Weight {
    pub fn resolve(&self) -> i32 {
        match self {
            Weight::Num(n) => *n as i32,
            Weight::Kw(s) => match s.as_str() {
                "normal" => 400,
                "bold" => 700,
                "lighter" => 300,
                "bolder" => 700,
                other => other.parse().unwrap_or(400),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Background {
    /// A CSS colour or gradient string.
    Css(String),
    Photo(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListStyle {
    /// `disc` | `circle` | `square` | `decimal` | `dash` | `none` | literal text.
    Name(String),
    /// A styled marker; `{}` is replaced with the item number.
    Span(Box<Node>),
}

/// `null` must survive as `Some(None)`: an explicit null clears a decoration
/// colour, which is different from the property being absent.
fn some_option<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(d).map(Some)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Props {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    // ── flexbox ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_content: Option<AlignContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<AlignItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<AlignItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub box_sizing: Option<BoxSizing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_basis: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<FlexDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_shrink: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_wrap: Option<FlexWrap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<JustifyContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow: Option<Overflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_gap: Option<f32>,

    // ── sizing ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<Dim>,

    // ── insets ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inset: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inset_inline: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inset_block: Option<Dim>,

    // ── margin ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_start: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_end: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_inline: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_block: Option<Dim>,

    // ── padding ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_start: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_end: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_inline: Option<Dim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_block: Option<Dim>,

    // ── border widths ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_top_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_right_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_bottom_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_left_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_start_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_end_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_inline_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_block_width: Option<f32>,

    // ── paint ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Vec<Background>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_smoothing: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner: Option<Corner>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadows: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<String>>,

    // ── pagination + grid placement ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_break: Option<PageBreak>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_column_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_column_span: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_row_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_row_span: Option<u32>,

    // ── grid tracks ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<GridTrack>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<GridTrack>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_rows: Option<Vec<GridTrack>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_columns: Option<Vec<GridTrack>>,

    // ── photo ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_aspect_ratio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_type: Option<ScaleType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_alignment: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flip_horizontal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flip_vertical: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip_path: Option<String>,

    // ── span/text style ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<FontStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<Weight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_spacing: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drop_shadows: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_dir: Option<Direction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<f32>,
    #[serde(
        default,
        deserialize_with = "some_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub underline_color: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_through: Option<f32>,
    #[serde(
        default,
        deserialize_with = "some_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub line_through_color: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overline: Option<f32>,
    #[serde(
        default,
        deserialize_with = "some_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub overline_color: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "some_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub highlight_color: Option<Option<String>>,

    // ── text block ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nowrap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_break: Option<LineBreakMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_overflow: Option<TextOverflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indent_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hanging_indent_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_stops: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_leader: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<TextAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_wrap: Option<TextWrap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autofit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_dir: Option<BaseDir>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip_image: Option<Box<Node>>,

    // ── path ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    /// Text outline width on spans; stroke width on paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_line_cap: Option<StrokeCap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_line_join: Option<StrokeJoin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_miter_limit: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_dash_array: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_dash_offset: Option<f32>,
    /// Path fill colour; also the Photo letterbox fill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_rule: Option<FillRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_path: Option<f32>,

    // ── table / list ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colspan: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rowspan: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_style: Option<ListStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker_gap: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker_offset: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i32>,
    /// Pre-resolved list marker, emitted by the dumper for callback markers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Box<Node>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeType {
    Column,
    Row,
    Grid,
    Text,
    Span,
    TextDefault,
    Photo,
    Path,
    Table,
    TableRow,
    TableCell,
    List,
    ListItem,
    ClipGroup,
}

/// A text node's inline content: raw text or a styled span.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Inline {
    Text(String),
    Span(Node),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "type")]
    pub ty: NodeTypeTag,
    #[serde(default)]
    pub props: Props,
    /// Container children. Empty for `text`/`span`, which use `inline`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Node>,
    /// Text/span content.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inline: Vec<Inline>,
}

/// Newtype so `Node: Default` works while `type` stays required on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeTypeTag(pub NodeType);

impl Default for NodeTypeTag {
    fn default() -> Self {
        NodeTypeTag(NodeType::Column)
    }
}

impl std::ops::Deref for NodeTypeTag {
    type Target = NodeType;
    fn deref(&self) -> &NodeType {
        &self.0
    }
}

impl Node {
    pub fn new(ty: NodeType) -> Self {
        Node {
            ty: NodeTypeTag(ty),
            ..Default::default()
        }
    }
    pub fn ty(&self) -> NodeType {
        self.ty.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontSpec {
    pub name: String,
    pub src: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LastPageHeight {
    Uniform,
    Content,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Margin {
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub right: f32,
    #[serde(default)]
    pub bottom: f32,
    #[serde(default)]
    pub left: f32,
}

impl<'de> serde::de::Deserialize<'de> for MarginSpec {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            All(f32),
            Sides(Margin),
        }
        Ok(MarginSpec(match Raw::deserialize(d)? {
            Raw::All(v) => Margin {
                top: v,
                right: v,
                bottom: v,
                left: v,
            },
            Raw::Sides(m) => m,
        }))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct MarginSpec(pub Margin);

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<MarginSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_page_height: Option<LastPageHeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Box<Node>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<Box<Node>>,
}

/// The whole document: schema version, config, fonts and the root node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub sone: u32,
    #[serde(default)]
    pub config: RenderConfig,
    #[serde(default)]
    pub fonts: Vec<FontSpec>,
    pub root: Node,
}

pub const IR_VERSION: u32 = 1;

impl Document {
    pub fn from_json(json: &str) -> crate::Result<Document> {
        let doc: Document = serde_json::from_str(json)
            .map_err(|e| crate::SoneError::ir(format!("/ (line {})", e.line()), e))?;
        if doc.sone != IR_VERSION {
            return Err(crate::SoneError::ir(
                "/sone",
                format!("unsupported IR version {}", doc.sone),
            ));
        }
        Ok(doc)
    }

    /// Strict mode additionally rejects unknown fields.
    pub fn from_json_strict(json: &str) -> crate::Result<Document> {
        let mut de = serde_json::Deserializer::from_str(json);
        let doc: Document = serde_path_to_error::deserialize(&mut de)
            .map_err(|e| crate::SoneError::ir(e.path().to_string(), e.into_inner()))?;
        if doc.sone != IR_VERSION {
            return Err(crate::SoneError::ir(
                "/sone",
                format!("unsupported IR version {}", doc.sone),
            ));
        }
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let json = r#"{
          "sone": 1,
          "config": { "density": 2, "pageHeight": 1123, "margin": 20 },
          "fonts": [{ "name": "Moul", "src": "file:./fonts/Moul.ttf" }],
          "root": {
            "type": "column",
            "props": { "gap": 8, "width": "50%", "padding": 12, "background": ["red"] },
            "children": [
              { "type": "text", "props": { "size": 14, "align": "center" },
                "inline": ["hello ", { "type": "span", "props": { "weight": "bold" }, "inline": ["world"] }] }
            ]
          }
        }"#;
        let doc = Document::from_json(json).unwrap();
        assert_eq!(doc.root.ty(), NodeType::Column);
        assert_eq!(doc.root.props.width, Some(Dim::Percent(50.0)));
        assert_eq!(doc.config.margin.unwrap().0.left, 20.0);

        let out = serde_json::to_string(&doc).unwrap();
        let again: Document = serde_json::from_str(&out).unwrap();
        assert_eq!(doc, again);
        assert_eq!(out, serde_json::to_string(&again).unwrap());
    }

    #[test]
    fn dim_forms() {
        let p: Props =
            serde_json::from_str(r#"{"width":"auto","height":100,"maxWidth":"25%"}"#).unwrap();
        assert_eq!(p.width, Some(Dim::Auto));
        assert_eq!(p.height, Some(Dim::Px(100.0)));
        assert_eq!(p.max_width, Some(Dim::Percent(25.0)));
    }

    #[test]
    fn grid_tracks() {
        let p: Props = serde_json::from_str(r#"{"columns":[100,"auto","1.5fr"]}"#).unwrap();
        assert_eq!(
            p.columns.unwrap(),
            vec![GridTrack::Fixed(100.0), GridTrack::Auto, GridTrack::Fr(1.5)]
        );
        assert!(serde_json::from_str::<Props>(r#"{"columns":["bogus"]}"#).is_err());
    }

    #[test]
    fn nullable_decoration_colors() {
        let p: Props =
            serde_json::from_str(r#"{"underlineColor":null,"overlineColor":"red"}"#).unwrap();
        assert_eq!(p.underline_color, Some(None));
        assert_eq!(p.overline_color, Some(Some("red".into())));
    }

    #[test]
    fn version_is_checked() {
        assert!(Document::from_json(r#"{"sone":99,"root":{"type":"column"}}"#).is_err());
    }

    #[test]
    fn strict_reports_a_path() {
        let e = Document::from_json_strict(r#"{"sone":1,"root":{"type":"nope"}}"#).unwrap_err();
        assert!(format!("{e}").contains("root.type"), "{e}");
    }
}
