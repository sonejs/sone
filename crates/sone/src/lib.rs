//! A declarative canvas layout engine with rich international text.
//!
//! ```no_run
//! use sone::prelude::*;
//!
//! # fn main() -> sone::Result<()> {
//! font("Inter", "fonts/Inter-Regular.ttf")?;
//!
//! let root = column()
//!     .gap(20)
//!     .padding(20)
//!     .size(420, 300)
//!     .bg("khaki")
//!     .corner_radius(28)
//!     .child(
//!         column()
//!             .flex(1)
//!             .corner_radius(20)
//!             .corner_smoothing(0.7)
//!             .bg("white"),
//!     )
//!     .child(
//!         row()
//!             .gap(10)
//!             .child(column().bg("lightgreen").square(50).corner_radius(14))
//!             .child(column().bg("salmon").height(50).corner_radius(14).flex(1)),
//!     );
//!
//! render(root).density(2).save("card.png")?;
//! # Ok(())
//! # }
//! ```
//!
//! This is the engine's own crate rather than a binding: there is no FFI in the
//! path, no IR string to parse, and no marshalling. The builder produces
//! [`sone_core::ir::Node`] directly and rendering hands it straight to Skia.

#[macro_use]
mod props;

mod nodes;
pub mod value;

#[cfg(feature = "render")]
mod render;

use sone_core::ir::{self, Node};

pub use nodes::*;
pub use sone_core::text::bidi::BaseDir;
pub use sone_core::ir::{
    AlignContent, AlignItems, BoxSizing, Corner, Dim, Direction, Display, FillRule,
    FlexDirection, FlexWrap, FontStyle, GridTrack, JustifyContent, LastPageHeight, LineBreakMode,
    Overflow, PageBreak, Position, ScaleType, StrokeCap, StrokeJoin, TextAlign, TextOverflow,
    TextWrap, Weight,
};
pub use sone_core::{Result, SoneError};

#[cfg(feature = "render")]
pub use render::{font, render, Engine, Granularity, Rendering};

/// Anything that can become an IR node.
pub trait IntoNode {
    fn into_node(self) -> Node;
}

impl IntoNode for Node {
    fn into_node(self) -> Node {
        self
    }
}

// ── factories ───────────────────────────────────────────────────────────────

/// A vertical container.
pub fn column() -> Column {
    Column::new()
}

/// A horizontal container.
pub fn row() -> Row {
    Row::new()
}

/// A grid container with row-major auto placement.
pub fn grid() -> Grid {
    Grid::new()
}

/// A paragraph.
pub fn text(content: impl Into<String>) -> Text {
    Text::new().content(content)
}

/// An empty paragraph, for one built entirely out of spans.
pub fn text_empty() -> Text {
    Text::new()
}

/// A styled run inside a [`Text`].
pub fn span(content: impl Into<String>) -> Span {
    Span::new().content(content)
}

/// Cascade text styling onto every descendant.
pub fn text_default() -> TextDefault {
    TextDefault::new()
}

/// An image, from a path, a URL, or `asset:name`.
pub fn photo(src: impl Into<String>) -> Photo {
    let mut node = Photo::new();
    node.node.props.src = Some(src.into());
    node
}

/// An image from raw bytes, inlined into the document as a data URL.
pub fn photo_bytes(data: &[u8]) -> Photo {
    use std::fmt::Write;
    // A small base64 encoder, so the crate needs no dependency for one call.
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let triple = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                let _ = write!(encoded, "{}", ALPHABET[((triple >> (18 - i * 6)) & 0x3f) as usize] as char);
            } else {
                encoded.push('=');
            }
        }
    }
    photo(format!("data:application/octet-stream;base64,{encoded}"))
}

/// An SVG path.
pub fn svg_path(d: impl Into<String>) -> SvgPath {
    let mut node = SvgPath::new();
    node.node.props.d = Some(d.into());
    node
}

/// A table. Children are rows.
pub fn table() -> Table {
    Table::new()
}

/// A table row. Children are cells.
pub fn table_row() -> TableRow {
    TableRow::new()
}

/// A table cell.
pub fn table_cell() -> TableCell {
    TableCell::new()
}

/// A bulleted or numbered list. Named `bullets` because `list` reads as a
/// collection everywhere else in Rust.
pub fn bullets() -> Bullets {
    Bullets::new()
}

/// One item in a list.
pub fn list_item() -> ListItem {
    ListItem::new()
}

/// Clip every child to an SVG path.
pub fn clip_group(path: impl Into<String>) -> ClipGroup {
    let mut node = ClipGroup::new();
    node.node.props.clip_path = Some(path.into());
    node
}

/// An explicit page break. Only meaningful with a page height set.
pub fn page_break() -> Column {
    column().height(0).page_break(ir::PageBreak::Before)
}

/// Everything a document needs, in one import.
pub mod prelude {
    pub use crate::{
        bullets, clip_group, column, grid, list_item, page_break, photo, photo_bytes, row, span,
        svg_path, table, table_cell, table_row, text, text_default, text_empty, IntoNode,
    };
    // The types too, so a helper function can name what it returns.
    pub use crate::{
        Bullets, ClipGroup, Column, Grid, ListItem, Photo, Row, Span, SvgPath, Table, TableCell,
        TableRow, Text, TextDefault,
    };
    pub use crate::{
        AlignContent, AlignItems, BaseDir, Corner, Dim, Direction, FillRule, FlexDirection,
        FlexWrap, FontStyle, GridTrack, JustifyContent, LastPageHeight, LineBreakMode, PageBreak,
        ScaleType, StrokeCap, StrokeJoin, TextAlign, TextOverflow, TextWrap, Weight,
    };

    #[cfg(feature = "render")]
    pub use crate::{font, render, Engine, Granularity};
}
