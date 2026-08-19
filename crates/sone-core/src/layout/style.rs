use taffy::geometry::Point;
use taffy::prelude::*;
use taffy::style::{BoxSizing, Overflow, Position};

use crate::ir::{self, Dim, Props};

/// Yoga's edge precedence: start/end, then side, then axis, then `all`.
fn edge<T: Copy>(
    side: Option<T>,
    start_end: Option<T>,
    axis: Option<T>,
    all: Option<T>,
) -> Option<T> {
    start_end.or(side).or(axis).or(all)
}

fn dim(d: Dim) -> Dimension {
    match d {
        Dim::Px(v) => length(v),
        Dim::Percent(v) => percent(v / 100.0f32),
        Dim::Auto => Dimension::auto(),
    }
}

fn lp(d: Dim) -> LengthPercentage {
    match d {
        Dim::Px(v) => length(v),
        Dim::Percent(v) => percent(v / 100.0f32),
        Dim::Auto => length(0.0f32),
    }
}

fn lpa(d: Dim) -> LengthPercentageAuto {
    match d {
        Dim::Px(v) => length(v),
        Dim::Percent(v) => percent(v / 100.0f32),
        Dim::Auto => LengthPercentageAuto::auto(),
    }
}

fn align_items(v: ir::AlignItems) -> AlignItems {
    match v {
        ir::AlignItems::FlexStart => AlignItems::FlexStart,
        ir::AlignItems::FlexEnd => AlignItems::FlexEnd,
        ir::AlignItems::Center => AlignItems::Center,
        ir::AlignItems::Stretch => AlignItems::Stretch,
        ir::AlignItems::Baseline => AlignItems::Baseline,
    }
}

fn align_content(v: ir::AlignContent) -> AlignContent {
    match v {
        ir::AlignContent::FlexStart => AlignContent::FlexStart,
        ir::AlignContent::FlexEnd => AlignContent::FlexEnd,
        ir::AlignContent::Center => AlignContent::Center,
        ir::AlignContent::Stretch => AlignContent::Stretch,
        ir::AlignContent::SpaceBetween => AlignContent::SpaceBetween,
        ir::AlignContent::SpaceAround => AlignContent::SpaceAround,
        ir::AlignContent::SpaceEvenly => AlignContent::SpaceEvenly,
    }
}

fn justify_content(v: ir::JustifyContent) -> JustifyContent {
    match v {
        ir::JustifyContent::FlexStart => JustifyContent::FlexStart,
        ir::JustifyContent::FlexEnd => JustifyContent::FlexEnd,
        ir::JustifyContent::Center => JustifyContent::Center,
        ir::JustifyContent::SpaceBetween => JustifyContent::SpaceBetween,
        ir::JustifyContent::SpaceAround => JustifyContent::SpaceAround,
        ir::JustifyContent::SpaceEvenly => JustifyContent::SpaceEvenly,
    }
}

/// Map sone layout props onto a taffy style, starting from Yoga's defaults
/// rather than taffy's (column direction, no shrink, flex-start align-content).
pub fn taffy_style(p: &Props) -> Style {
    let mut s = Style {
        // Yoga defaults, not taffy's.
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        align_content: Some(AlignContent::FlexStart),
        ..Default::default()
    };

    if let Some(v) = p.align_content {
        s.align_content = Some(align_content(v))
    }
    if let Some(v) = p.align_items {
        s.align_items = Some(align_items(v))
    }
    if let Some(v) = p.align_self {
        s.align_self = Some(align_items(v))
    }
    if let Some(v) = p.justify_content {
        s.justify_content = Some(justify_content(v))
    }
    s.aspect_ratio = p.aspect_ratio;

    if let Some(v) = p.box_sizing {
        s.box_sizing = match v {
            ir::BoxSizing::BorderBox => BoxSizing::BorderBox,
            ir::BoxSizing::ContentBox => BoxSizing::ContentBox,
        };
    }
    if let Some(v) = p.display {
        s.display = match v {
            ir::Display::None => Display::None,
            // taffy has no `contents`; the box still participates in flex.
            ir::Display::Flex | ir::Display::Contents => Display::Flex,
        };
    }
    if let Some(v) = p.flex_direction {
        s.flex_direction = match v {
            ir::FlexDirection::Row => FlexDirection::Row,
            ir::FlexDirection::Column => FlexDirection::Column,
            ir::FlexDirection::RowReverse => FlexDirection::RowReverse,
            ir::FlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        };
    }
    if let Some(v) = p.flex_wrap {
        s.flex_wrap = match v {
            ir::FlexWrap::Wrap => FlexWrap::Wrap,
            ir::FlexWrap::NoWrap => FlexWrap::NoWrap,
            ir::FlexWrap::WrapReverse => FlexWrap::WrapReverse,
        };
    }
    if let Some(v) = p.overflow {
        let o = match v {
            ir::Overflow::Visible => Overflow::Visible,
            ir::Overflow::Hidden => Overflow::Hidden,
            ir::Overflow::Scroll => Overflow::Scroll,
        };
        s.overflow = Point { x: o, y: o };
    }
    if let Some(v) = p.position {
        s.position = match v {
            ir::Position::Absolute => Position::Absolute,
            // Yoga's `static` has no taffy equivalent; relative is the closest.
            ir::Position::Relative | ir::Position::Static => Position::Relative,
        };
    }

    // Yoga's `flex` shorthand: grow, shrink and basis in one value.
    if let Some(v) = p.flex {
        s.flex_grow = v.max(0.0);
        s.flex_shrink = if v < 0.0 { -v } else { 0.0 };
        s.flex_basis = if v > 0.0 {
            length(0.0f32)
        } else {
            Dimension::auto()
        };
    }
    if let Some(v) = p.flex_grow {
        s.flex_grow = v
    }
    if let Some(v) = p.flex_shrink {
        s.flex_shrink = v
    }
    if let Some(v) = p.flex_basis {
        s.flex_basis = dim(v)
    }

    let gap = p.gap.unwrap_or(0.0);
    s.gap = Size {
        width: length(p.column_gap.unwrap_or(gap)),
        height: length(p.row_gap.unwrap_or(gap)),
    };

    s.size = Size {
        width: p.width.map(dim).unwrap_or(Dimension::auto()),
        height: p.height.map(dim).unwrap_or(Dimension::auto()),
    };
    s.min_size = Size {
        width: p.min_width.map(dim).unwrap_or(Dimension::auto()),
        height: p.min_height.map(dim).unwrap_or(Dimension::auto()),
    };
    s.max_size = Size {
        width: p.max_width.map(dim).unwrap_or(Dimension::auto()),
        height: p.max_height.map(dim).unwrap_or(Dimension::auto()),
    };

    s.margin = Rect {
        left: edge(p.margin_left, p.margin_start, p.margin_inline, p.margin)
            .map(lpa)
            .unwrap_or(length(0.0f32)),
        right: edge(p.margin_right, p.margin_end, p.margin_inline, p.margin)
            .map(lpa)
            .unwrap_or(length(0.0f32)),
        top: edge(p.margin_top, None, p.margin_block, p.margin)
            .map(lpa)
            .unwrap_or(length(0.0f32)),
        bottom: edge(p.margin_bottom, None, p.margin_block, p.margin)
            .map(lpa)
            .unwrap_or(length(0.0f32)),
    };

    s.padding = Rect {
        left: edge(p.padding_left, p.padding_start, p.padding_inline, p.padding)
            .map(lp)
            .unwrap_or(length(0.0f32)),
        right: edge(p.padding_right, p.padding_end, p.padding_inline, p.padding)
            .map(lp)
            .unwrap_or(length(0.0f32)),
        top: edge(p.padding_top, None, p.padding_block, p.padding)
            .map(lp)
            .unwrap_or(length(0.0f32)),
        bottom: edge(p.padding_bottom, None, p.padding_block, p.padding)
            .map(lp)
            .unwrap_or(length(0.0f32)),
    };

    let bw = |v: Option<f32>| v.map(Dim::Px);
    s.border = Rect {
        left: edge(
            bw(p.border_left_width),
            bw(p.border_start_width),
            bw(p.border_inline_width),
            bw(p.border_width),
        )
        .map(lp)
        .unwrap_or(length(0.0f32)),
        right: edge(
            bw(p.border_right_width),
            bw(p.border_end_width),
            bw(p.border_inline_width),
            bw(p.border_width),
        )
        .map(lp)
        .unwrap_or(length(0.0f32)),
        top: edge(
            bw(p.border_top_width),
            None,
            bw(p.border_block_width),
            bw(p.border_width),
        )
        .map(lp)
        .unwrap_or(length(0.0f32)),
        bottom: edge(
            bw(p.border_bottom_width),
            None,
            bw(p.border_block_width),
            bw(p.border_width),
        )
        .map(lp)
        .unwrap_or(length(0.0f32)),
    };

    s.inset = Rect {
        left: edge(p.left, p.start, p.inset_inline, p.inset)
            .map(lpa)
            .unwrap_or(LengthPercentageAuto::auto()),
        right: edge(p.right, p.end, p.inset_inline, p.inset)
            .map(lpa)
            .unwrap_or(LengthPercentageAuto::auto()),
        top: edge(p.top, None, p.inset_block, p.inset)
            .map(lpa)
            .unwrap_or(LengthPercentageAuto::auto()),
        bottom: edge(p.bottom, None, p.inset_block, p.inset)
            .map(lpa)
            .unwrap_or(LengthPercentageAuto::auto()),
    };

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Props;

    #[test]
    fn yoga_defaults_are_applied() {
        let s = taffy_style(&Props::default());
        assert_eq!(s.flex_direction, FlexDirection::Column);
        assert_eq!(s.flex_shrink, 0.0);
        assert_eq!(s.align_content, Some(AlignContent::FlexStart));
    }

    #[test]
    fn flex_shorthand_matches_yoga() {
        let mut p = Props {
            flex: Some(2.0),
            ..Default::default()
        };
        let s = taffy_style(&p);
        assert_eq!(s.flex_grow, 2.0);
        assert_eq!(s.flex_shrink, 0.0);
        assert_eq!(s.flex_basis, length(0.0f32));

        p.flex = Some(-3.0);
        let s = taffy_style(&p);
        assert_eq!(s.flex_grow, 0.0);
        assert_eq!(s.flex_shrink, 3.0);
        assert_eq!(s.flex_basis, Dimension::auto());
    }

    #[test]
    fn edges_follow_yoga_precedence() {
        let mut p = Props {
            padding: Some(Dim::Px(4.0)),
            padding_inline: Some(Dim::Px(8.0)),
            padding_left: Some(Dim::Px(12.0)),
            ..Default::default()
        };
        let s = taffy_style(&p);
        assert_eq!(s.padding.left, length(12.0_f32));
        assert_eq!(s.padding.right, length(8.0_f32));
        assert_eq!(s.padding.top, length(4.0_f32));

        p.padding_start = Some(Dim::Px(20.0));
        assert_eq!(taffy_style(&p).padding.left, length(20.0_f32));
    }

    #[test]
    fn percentages_convert_to_ratios() {
        let p = Props {
            width: Some(Dim::Percent(50.0)),
            ..Default::default()
        };
        assert_eq!(taffy_style(&p).size.width, percent(0.5_f32));
    }

    #[test]
    fn gap_falls_back_to_the_shorthand() {
        let p = Props {
            gap: Some(10.0),
            column_gap: Some(4.0),
            ..Default::default()
        };
        let s = taffy_style(&p);
        assert_eq!(s.gap.width, length(4.0_f32));
        assert_eq!(s.gap.height, length(10.0_f32));
    }
}
