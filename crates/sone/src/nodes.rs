//! The node types.
//!
//! One struct per IR node type, so the compiler can tell them apart. That is
//! what lets `Text::size` mean the font size while `Column::size` means the
//! box — the ambiguity every other binding had to resolve by hand.

use sone_core::ir::{self, Node, NodeType, NodeTypeTag};

use crate::IntoNode;

macro_rules! node_types {
    ($( $(#[$meta:meta])* $ty:ident => $variant:ident ),* $(,)?) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone)]
            pub struct $ty {
                pub(crate) node: Node,
            }

            impl $ty {
                pub(crate) fn new() -> Self {
                    $ty {
                        node: Node {
                            ty: NodeTypeTag(NodeType::$variant),
                            ..Default::default()
                        },
                    }
                }
            }

            impl IntoNode for $ty {
                fn into_node(self) -> Node {
                    self.node
                }
            }

            impl From<$ty> for Node {
                fn from(value: $ty) -> Node {
                    value.node
                }
            }
        )*
    };
}

node_types! {
    /// A vertical container.
    Column => Column,
    /// A horizontal container.
    Row => Row,
    /// A grid container with row-major auto placement.
    Grid => Grid,
    /// A paragraph of text and styled runs.
    Text => Text,
    /// A styled run inside a [`Text`].
    Span => Span,
    /// Cascades text styling onto its descendants without drawing a box.
    TextDefault => TextDefault,
    /// An image.
    Photo => Photo,
    /// An SVG path.
    SvgPath => Path,
    /// A table. Children are rows.
    Table => Table,
    /// A table row. Children are cells.
    TableRow => TableRow,
    /// A table cell.
    TableCell => TableCell,
    /// A bulleted or numbered list. Children are items.
    Bullets => List,
    /// One item in a [`Bullets`] list.
    ListItem => ListItem,
    /// Clips every child to an SVG path.
    ClipGroup => ClipGroup,
}

impl_layout_props!(
    Column, Row, Grid, Text, Photo, SvgPath, Table, TableRow, TableCell, Bullets, ListItem,
    ClipGroup,
);
impl_box_size!(
    Column, Row, Grid, Photo, SvgPath, Table, TableRow, TableCell, Bullets, ListItem, ClipGroup,
);
impl_span_props!(Text, Span, TextDefault);
impl_text_block_props!(Text, TextDefault);
impl_children!(
    Column, Row, Grid, TextDefault, Table, TableRow, TableCell, Bullets, ListItem, ClipGroup,
);

impl Text {
    /// The **font** size, not the box size — matching the TypeScript API, where
    /// a text builder omits the layout `size`. Use `width` and `height` for the
    /// box; they are still here, because `Text` is a box too.
    pub fn size(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.size = Some(value.as_f32());
        self
    }

    /// Append raw text.
    pub fn content(mut self, text: impl Into<String>) -> Self {
        self.node.inline.push(ir::Inline::Text(text.into()));
        self
    }

    /// Append a styled run.
    pub fn span(mut self, span: Span) -> Self {
        self.node.inline.push(ir::Inline::Span(span.into_node()));
        self
    }
}

impl Span {
    /// The font size.
    pub fn size(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.size = Some(value.as_f32());
        self
    }

    /// Append raw text.
    pub fn content(mut self, text: impl Into<String>) -> Self {
        self.node.inline.push(ir::Inline::Text(text.into()));
        self
    }
}

impl TextDefault {
    /// The font size cascaded onto descendants.
    pub fn size(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.size = Some(value.as_f32());
        self
    }
}

impl Grid {
    pub fn columns(mut self, tracks: impl IntoIterator<Item = ir::GridTrack>) -> Self {
        self.node.props.columns = Some(tracks.into_iter().collect());
        self
    }

    pub fn rows(mut self, tracks: impl IntoIterator<Item = ir::GridTrack>) -> Self {
        self.node.props.rows = Some(tracks.into_iter().collect());
        self
    }

    pub fn auto_rows(mut self, tracks: impl IntoIterator<Item = ir::GridTrack>) -> Self {
        self.node.props.auto_rows = Some(tracks.into_iter().collect());
        self
    }

    pub fn auto_columns(mut self, tracks: impl IntoIterator<Item = ir::GridTrack>) -> Self {
        self.node.props.auto_columns = Some(tracks.into_iter().collect());
        self
    }
}

impl Photo {
    /// How the image fills its box.
    pub fn scale_type(mut self, value: ir::ScaleType) -> Self {
        self.node.props.scale_type = Some(value);
        self
    }

    /// 0 is start, 0.5 centre, 1 end.
    pub fn scale_alignment(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.scale_alignment = Some(value.as_f32());
        self
    }

    pub fn preserve_aspect_ratio(mut self, value: bool) -> Self {
        self.node.props.preserve_aspect_ratio = Some(value);
        self
    }

    pub fn flip_horizontal(mut self, value: bool) -> Self {
        self.node.props.flip_horizontal = Some(value);
        self
    }

    pub fn flip_vertical(mut self, value: bool) -> Self {
        self.node.props.flip_vertical = Some(value);
        self
    }

    /// The letterbox colour behind a `contain` image.
    pub fn fill(mut self, color: impl Into<String>) -> Self {
        self.node.props.fill = Some(color.into());
        self
    }

    /// An SVG path the image is clipped to.
    pub fn clip_path(mut self, path: impl Into<String>) -> Self {
        self.node.props.clip_path = Some(path.into());
        self
    }
}

impl SvgPath {
    pub fn stroke(mut self, color: impl Into<String>) -> Self {
        self.node.props.stroke = Some(color.into());
        self
    }

    pub fn stroke_width(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.stroke_width = Some(value.as_f32());
        self
    }

    pub fn stroke_line_cap(mut self, value: ir::StrokeCap) -> Self {
        self.node.props.stroke_line_cap = Some(value);
        self
    }

    pub fn stroke_line_join(mut self, value: ir::StrokeJoin) -> Self {
        self.node.props.stroke_line_join = Some(value);
        self
    }

    pub fn stroke_miter_limit(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.stroke_miter_limit = Some(value.as_f32());
        self
    }

    pub fn stroke_dash_array(mut self, values: impl IntoIterator<Item = f32>) -> Self {
        self.node.props.stroke_dash_array = Some(values.into_iter().collect());
        self
    }

    pub fn stroke_dash_offset(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.stroke_dash_offset = Some(value.as_f32());
        self
    }

    pub fn fill(mut self, color: impl Into<String>) -> Self {
        self.node.props.fill = Some(color.into());
        self
    }

    pub fn fill_opacity(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.fill_opacity = Some(value.as_f32());
        self
    }

    pub fn fill_rule(mut self, value: ir::FillRule) -> Self {
        self.node.props.fill_rule = Some(value);
        self
    }

    /// Scale the path data itself, before layout.
    pub fn scale_path(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.scale_path = Some(value.as_f32());
        self
    }
}

impl Table {
    /// Row and column spacing.
    pub fn spacing(mut self, row: impl crate::value::Num, column: impl crate::value::Num) -> Self {
        self.node.props.spacing = Some(vec![row.as_f32(), column.as_f32()]);
        self
    }
}

impl TableCell {
    pub fn colspan(mut self, value: impl crate::value::Int) -> Self {
        self.node.props.colspan = Some(value.as_i64() as u32);
        self
    }

    pub fn rowspan(mut self, value: impl crate::value::Int) -> Self {
        self.node.props.rowspan = Some(value.as_i64() as u32);
        self
    }
}

impl Bullets {
    /// `"disc"`, `"decimal"`, literal text, or a styled marker node.
    pub fn list_style(mut self, value: impl Into<ir::ListStyle>) -> Self {
        self.node.props.list_style = Some(value.into());
        self
    }

    pub fn marker_gap(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.marker_gap = Some(value.as_f32());
        self
    }

    pub fn marker_offset(mut self, value: impl crate::value::Num) -> Self {
        self.node.props.marker_offset = Some(value.as_f32());
        self
    }

    pub fn start_index(mut self, value: impl crate::value::Int) -> Self {
        self.node.props.start_index = Some(value.as_i64() as i32);
        self
    }
}

impl ListItem {
    /// Override the list's marker for this item alone.
    pub fn marker(mut self, marker: impl IntoNode) -> Self {
        self.node.props.marker = Some(Box::new(marker.into_node()));
        self
    }
}
