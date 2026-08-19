//! The property setters, generated once and expanded per node type.
//!
//! Rust has no inheritance, so the shared-properties problem gets the same
//! answer Ruby's class macros and C#'s generic extensions did: write the list
//! once and expand it into every type that carries it. Because the types stay
//! distinct, `Text::size` can mean the font size while `Column::size` means the
//! box — the collision every other binding had to resolve by hand does not
//! arise here.

/// One field, one setter.
macro_rules! setters {
    ($( $(#[$meta:meta])* $name:ident => $field:ident : $arg:ty ),* $(,)?) => {
        $(
            $(#[$meta])*
            pub fn $name(mut self, value: impl Into<$arg>) -> Self {
                self.node.props.$field = Some(value.into());
                self
            }
        )*
    };
}

/// One field, one setter, taking any number.
macro_rules! number_setters {
    ($( $(#[$meta:meta])* $name:ident => $field:ident ),* $(,)?) => {
        $(
            $(#[$meta])*
            pub fn $name(mut self, value: impl $crate::value::Num) -> Self {
                self.node.props.$field = Some(value.as_f32());
                self
            }
        )*
    };
}

macro_rules! impl_layout_props {
    ($($ty:ident),+ $(,)?) => {$(
        impl $ty {
            setters! {
                /// A name for this node, echoed back by `layout` and `metadata`.
                tag => tag: String,
                align_content => align_content: ::sone_core::ir::AlignContent,
                align_items => align_items: ::sone_core::ir::AlignItems,
                align_self => align_self: ::sone_core::ir::AlignItems,
                box_sizing => box_sizing: ::sone_core::ir::BoxSizing,
                direction => direction: ::sone_core::ir::Direction,
                display => display: ::sone_core::ir::Display,
                flex_direction => flex_direction: ::sone_core::ir::FlexDirection,
                /// Flexbox wrapping. On `Text`, `wrap` is the paragraph one.
                wrap => flex_wrap: ::sone_core::ir::FlexWrap,
                justify_content => justify_content: ::sone_core::ir::JustifyContent,
                overflow => overflow: ::sone_core::ir::Overflow,
                position => position: ::sone_core::ir::Position,
                /// The flex base size, before grow and shrink.
                basis => flex_basis: ::sone_core::ir::Dim,
                width => width: ::sone_core::ir::Dim,
                height => height: ::sone_core::ir::Dim,
                min_width => min_width: ::sone_core::ir::Dim,
                min_height => min_height: ::sone_core::ir::Dim,
                max_width => max_width: ::sone_core::ir::Dim,
                max_height => max_height: ::sone_core::ir::Dim,
                margin_top => margin_top: ::sone_core::ir::Dim,
                margin_right => margin_right: ::sone_core::ir::Dim,
                margin_bottom => margin_bottom: ::sone_core::ir::Dim,
                margin_left => margin_left: ::sone_core::ir::Dim,
                padding_top => padding_top: ::sone_core::ir::Dim,
                padding_right => padding_right: ::sone_core::ir::Dim,
                padding_bottom => padding_bottom: ::sone_core::ir::Dim,
                padding_left => padding_left: ::sone_core::ir::Dim,
                top => top: ::sone_core::ir::Dim,
                right => right: ::sone_core::ir::Dim,
                bottom => bottom: ::sone_core::ir::Dim,
                left => left: ::sone_core::ir::Dim,
                /// The leading inset, which flips with the writing direction.
                start => start: ::sone_core::ir::Dim,
                /// The trailing inset, which flips with the writing direction.
                end => end: ::sone_core::ir::Dim,
                inset => inset: ::sone_core::ir::Dim,
                border_color => border_color: String,
                corner => corner: ::sone_core::ir::Corner,
                /// Force or forbid a page break here. Needs `page_height`.
                page_break => page_break: ::sone_core::ir::PageBreak,
            }

            number_setters! {
                flex => flex,
                grow => flex_grow,
                shrink => flex_shrink,
                aspect_ratio => aspect_ratio,
                gap => gap,
                row_gap => row_gap,
                column_gap => column_gap,
                opacity => opacity,
                /// Squircle-ness, 0..1. Figma's corner smoothing.
                corner_smoothing => corner_smoothing,
                /// Rotation in degrees, about the node's centre.
                rotate => rotation,
                translate_x => translate_x,
                translate_y => translate_y,
            }

            /// All four sides.
            pub fn padding(mut self, value: impl Into<::sone_core::ir::Dim>) -> Self {
                self.node.props.padding = Some(value.into());
                self
            }

            /// Top, right, bottom, left — the CSS order.
            pub fn padding_each(
                self,
                top: impl Into<::sone_core::ir::Dim>,
                right: impl Into<::sone_core::ir::Dim>,
                bottom: impl Into<::sone_core::ir::Dim>,
                left: impl Into<::sone_core::ir::Dim>,
            ) -> Self {
                self.padding_top(top)
                    .padding_right(right)
                    .padding_bottom(bottom)
                    .padding_left(left)
            }

            /// All four sides.
            pub fn margin(mut self, value: impl Into<::sone_core::ir::Dim>) -> Self {
                self.node.props.margin = Some(value.into());
                self
            }

            /// Top, right, bottom, left — the CSS order.
            pub fn margin_each(
                self,
                top: impl Into<::sone_core::ir::Dim>,
                right: impl Into<::sone_core::ir::Dim>,
                bottom: impl Into<::sone_core::ir::Dim>,
                left: impl Into<::sone_core::ir::Dim>,
            ) -> Self {
                self.margin_top(top)
                    .margin_right(right)
                    .margin_bottom(bottom)
                    .margin_left(left)
            }

            /// All four sides.
            pub fn border_width(mut self, value: impl $crate::value::Num) -> Self {
                self.node.props.border_width = Some(value.as_f32());
                self
            }

            /// Top, right, bottom, left — the CSS order.
            pub fn border_width_each(
                mut self,
                top: impl $crate::value::Num,
                right: impl $crate::value::Num,
                bottom: impl $crate::value::Num,
                left: impl $crate::value::Num,
            ) -> Self {
                self.node.props.border_top_width = Some(top.as_f32());
                self.node.props.border_right_width = Some(right.as_f32());
                self.node.props.border_bottom_width = Some(bottom.as_f32());
                self.node.props.border_left_width = Some(left.as_f32());
                self
            }

            /// Scale. One argument scales both axes.
            pub fn scale(mut self, x: impl $crate::value::Num, y: impl $crate::value::Num) -> Self {
                self.node.props.scale = Some([x.as_f32(), y.as_f32()]);
                self
            }

            /// Add a background layer: a CSS colour, a gradient, or a `Photo`.
            pub fn bg(mut self, layer: impl Into<::sone_core::ir::Background>) -> Self {
                self.node
                    .props
                    .background
                    .get_or_insert_with(Vec::new)
                    .push(layer.into());
                self
            }

            /// Corner radii: one value for all four, or up to four from the top left.
            pub fn corner_radius(mut self, radii: impl $crate::value::Radii) -> Self {
                self.node.props.corner_radius = Some(radii.into_vec());
                self
            }

            /// Add a CSS `box-shadow`.
            pub fn shadow(mut self, shadow: impl Into<String>) -> Self {
                self.node
                    .props
                    .shadows
                    .get_or_insert_with(Vec::new)
                    .push(shadow.into());
                self
            }

            /// Add a CSS filter. Applied in the order they are added.
            pub fn filter(mut self, filter: impl Into<String>) -> Self {
                self.node
                    .props
                    .filters
                    .get_or_insert_with(Vec::new)
                    .push(filter.into());
                self
            }

            pub fn blur(self, radius: impl $crate::value::Num) -> Self {
                self.filter(format!("blur({}px)", $crate::value::css(radius.as_f32())))
            }

            pub fn brightness(self, amount: impl $crate::value::Num) -> Self {
                self.filter(format!("brightness({})", $crate::value::css(amount.as_f32())))
            }

            pub fn contrast(self, amount: impl $crate::value::Num) -> Self {
                self.filter(format!("contrast({})", $crate::value::css(amount.as_f32())))
            }

            pub fn grayscale(self, amount: impl $crate::value::Num) -> Self {
                self.filter(format!("grayscale({})", $crate::value::css(amount.as_f32())))
            }

            pub fn hue_rotate(self, degrees: impl $crate::value::Num) -> Self {
                self.filter(format!("hue-rotate({})", $crate::value::css(degrees.as_f32())))
            }

            pub fn invert(self, amount: impl $crate::value::Num) -> Self {
                self.filter(format!("invert({})", $crate::value::css(amount.as_f32())))
            }

            pub fn saturate(self, amount: impl $crate::value::Num) -> Self {
                self.filter(format!("saturate({})", $crate::value::css(amount.as_f32())))
            }

            pub fn sepia(self, amount: impl $crate::value::Num) -> Self {
                self.filter(format!("sepia({})", $crate::value::css(amount.as_f32())))
            }

            /// Place in a grid column, optionally spanning several.
            pub fn grid_column(mut self, start: impl $crate::value::Int) -> Self {
                self.node.props.grid_column_start = Some(start.as_i64() as u32);
                self
            }

            pub fn grid_column_span(mut self, span: impl $crate::value::Int) -> Self {
                self.node.props.grid_column_span = Some(span.as_i64() as u32);
                self
            }

            pub fn grid_row(mut self, start: impl $crate::value::Int) -> Self {
                self.node.props.grid_row_start = Some(start.as_i64() as u32);
                self
            }

            pub fn grid_row_span(mut self, span: impl $crate::value::Int) -> Self {
                self.node.props.grid_row_span = Some(span.as_i64() as u32);
                self
            }

            /// Set raw IR properties, for anything this API does not cover yet.
            pub fn props(mut self, edit: impl FnOnce(&mut ::sone_core::ir::Props)) -> Self {
                edit(&mut self.node.props);
                self
            }
        }
    )+};
}

/// Box sizing, for every node except `Text` — there `size` is the font size.
macro_rules! impl_box_size {
    ($($ty:ident),+ $(,)?) => {$(
        impl $ty {
            /// Width and height. Use `square` for one value.
            pub fn size(
                self,
                width: impl Into<::sone_core::ir::Dim>,
                height: impl Into<::sone_core::ir::Dim>,
            ) -> Self {
                self.width(width).height(height)
            }

            /// Width and height set to the same value.
            pub fn square(self, side: impl Into<::sone_core::ir::Dim> + Copy) -> Self {
                self.width(side).height(side)
            }
        }
    )+};
}

macro_rules! impl_span_props {
    ($($ty:ident),+ $(,)?) => {$(
        impl $ty {
            setters! {
                color => color: String,
                style => style: ::sone_core::ir::FontStyle,
                /// A CSS keyword such as `"bold"`, or a number.
                weight => weight: ::sone_core::ir::Weight,
                /// The glyph outline colour.
                stroke_color => stroke_color: String,
                /// Force this run's direction, overriding bidi resolution.
                text_dir => text_dir: ::sone_core::ir::Direction,
            }

            number_setters! {
                letter_spacing => letter_spacing,
                word_spacing => word_spacing,
                /// The glyph outline width.
                stroke_width => stroke_width,
                /// Shift the run off its baseline — superscripts, subscripts.
                offset_y => offset_y,
            }

            /// The font stack, in fallback order.
            pub fn font<S: Into<String>>(mut self, families: impl IntoIterator<Item = S>) -> Self {
                self.node.props.font = Some(families.into_iter().map(Into::into).collect());
                self
            }

            /// A single font family.
            pub fn font_family(mut self, family: impl Into<String>) -> Self {
                self.node.props.font = Some(vec![family.into()]);
                self
            }

            pub fn underline(mut self, thickness: impl $crate::value::Num) -> Self {
                self.node.props.underline = Some(thickness.as_f32());
                self
            }

            pub fn overline(mut self, thickness: impl $crate::value::Num) -> Self {
                self.node.props.overline = Some(thickness.as_f32());
                self
            }

            pub fn line_through(mut self, thickness: impl $crate::value::Num) -> Self {
                self.node.props.line_through = Some(thickness.as_f32());
                self
            }

            /// `None` is an explicit null: use the text colour. The engine reads
            /// that differently from the property being absent, which is why it
            /// takes an `Option` rather than defaulting.
            pub fn underline_color(mut self, color: Option<impl Into<String>>) -> Self {
                self.node.props.underline_color = Some(color.map(Into::into));
                self
            }

            pub fn overline_color(mut self, color: Option<impl Into<String>>) -> Self {
                self.node.props.overline_color = Some(color.map(Into::into));
                self
            }

            pub fn line_through_color(mut self, color: Option<impl Into<String>>) -> Self {
                self.node.props.line_through_color = Some(color.map(Into::into));
                self
            }

            pub fn highlight(mut self, color: Option<impl Into<String>>) -> Self {
                self.node.props.highlight_color = Some(color.map(Into::into));
                self
            }

            /// Add a CSS `text-shadow`.
            pub fn drop_shadow(mut self, shadow: impl Into<String>) -> Self {
                self.node
                    .props
                    .drop_shadows
                    .get_or_insert_with(Vec::new)
                    .push(shadow.into());
                self
            }
        }
    )+};
}

macro_rules! impl_text_block_props {
    ($($ty:ident),+ $(,)?) => {$(
        impl $ty {
            setters! {
                line_break => line_break: ::sone_core::ir::LineBreakMode,
                text_overflow => text_overflow: ::sone_core::ir::TextOverflow,
                align => align: ::sone_core::ir::TextAlign,
                /// Greedy wrapping, or balancing for a ragged edge.
                text_wrap => text_wrap: ::sone_core::ir::TextWrap,
                /// The character filling the space a tab skips.
                tab_leader => tab_leader: String,
                /// The base direction used to resolve bidi runs.
                base_dir => base_dir: ::sone_core::text::bidi::BaseDir,
            }

            number_setters! {
                /// Truncate after this many lines.
                max_lines => max_lines,
                /// Line height as a multiple of the font size.
                line_height => line_height,
                /// First-line indent.
                indent => indent_size,
                /// Indent every line but the first.
                hanging_indent => hanging_indent_size,
            }

            /// Never wrap this paragraph.
            pub fn nowrap(mut self) -> Self {
                self.node.props.nowrap = Some(true);
                self
            }

            /// Whether the paragraph wraps. Not the flexbox `wrap`.
            pub fn wrap_text(mut self, wrap: bool) -> Self {
                self.node.props.nowrap = Some(!wrap);
                self
            }

            /// Shrink the text until it fits its box.
            pub fn autofit(mut self) -> Self {
                self.node.props.autofit = Some(true);
                self
            }

            /// Rotation of the text inside its box, in degrees.
            pub fn orientation(mut self, degrees: impl $crate::value::Int) -> Self {
                self.node.props.orientation = Some(degrees.as_i64() as u32);
                self
            }

            pub fn tab_stops(mut self, stops: impl IntoIterator<Item = f32>) -> Self {
                self.node.props.tab_stops = Some(stops.into_iter().collect());
                self
            }

            /// Paint the glyphs with an image instead of a colour.
            pub fn clip_image(mut self, photo: Photo) -> Self {
                self.node.props.clip_image = Some(Box::new(photo.into_node()));
                self
            }
        }
    )+};
}

macro_rules! impl_children {
    ($($ty:ident),+ $(,)?) => {$(
        impl $ty {
            /// Append one child.
            pub fn child(mut self, child: impl $crate::IntoNode) -> Self {
                self.node.children.push(child.into_node());
                self
            }

            /// Append many — the answer to generated content.
            pub fn children<T: $crate::IntoNode>(
                mut self,
                children: impl IntoIterator<Item = T>,
            ) -> Self {
                self.node
                    .children
                    .extend(children.into_iter().map(|child| child.into_node()));
                self
            }

            /// Append a child only when there is one.
            pub fn maybe_child(self, child: Option<impl $crate::IntoNode>) -> Self {
                match child {
                    Some(child) => self.child(child),
                    None => self,
                }
            }
        }
    )+};
}
