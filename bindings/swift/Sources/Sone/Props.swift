import Foundation

/// Flexbox, sizing, spacing and the visual box properties.
///
/// A class-constrained protocol, so `Self` in an extension is the concrete node
/// and `Column().bg("salmon").size(50)` stays a `Column` all the way down.
public protocol LayoutNode: Node {}

/// Span-level text styling.
public protocol SpanStyled: Node {}

/// Paragraph-level properties.
public protocol TextBlock: Node {}

extension LayoutNode {
    @discardableResult public func alignContent(_ value: AlignContent) -> Self { set("alignContent", .string(value.rawValue)) }
    @discardableResult public func alignItems(_ value: AlignItems) -> Self { set("alignItems", .string(value.rawValue)) }
    @discardableResult public func alignSelf(_ value: AlignItems) -> Self { set("alignSelf", .string(value.rawValue)) }
    @discardableResult public func aspectRatio(_ value: Double) -> Self { set("aspectRatio", .number(value)) }
    @discardableResult public func boxSizing(_ value: BoxSizing) -> Self { set("boxSizing", .string(value.rawValue)) }
    @discardableResult public func direction(_ value: WritingDirection) -> Self { set("direction", .string(value.rawValue)) }
    @discardableResult public func display(_ value: Display) -> Self { set("display", .string(value.rawValue)) }
    @discardableResult public func flex(_ value: Double) -> Self { set("flex", .number(value)) }
    @discardableResult public func basis(_ value: Dim) -> Self { set("flexBasis", value.ir) }
    @discardableResult public func flexDirection(_ value: FlexDirection) -> Self { set("flexDirection", .string(value.rawValue)) }
    @discardableResult public func grow(_ value: Double) -> Self { set("flexGrow", .number(value)) }
    @discardableResult public func shrink(_ value: Double) -> Self { set("flexShrink", .number(value)) }
    @discardableResult public func wrap(_ value: FlexWrap) -> Self { set("flexWrap", .string(value.rawValue)) }
    @discardableResult public func justifyContent(_ value: JustifyContent) -> Self { set("justifyContent", .string(value.rawValue)) }
    @discardableResult public func overflow(_ value: Overflow) -> Self { set("overflow", .string(value.rawValue)) }
    @discardableResult public func position(_ value: Position) -> Self { set("position", .string(value.rawValue)) }

    @discardableResult public func gap(_ value: Double) -> Self { set("gap", .number(value)) }
    @discardableResult public func rowGap(_ value: Double) -> Self { set("rowGap", .number(value)) }
    @discardableResult public func columnGap(_ value: Double) -> Self { set("columnGap", .number(value)) }

    /// Width and height. One argument makes a square.
    @discardableResult public func size(_ width: Dim, _ height: Dim? = nil) -> Self {
        props.set("width", width.ir)
        props.set("height", (height ?? width).ir)
        return self
    }

    @discardableResult public func width(_ value: Dim) -> Self { set("width", value.ir) }
    @discardableResult public func height(_ value: Dim) -> Self { set("height", value.ir) }
    @discardableResult public func minWidth(_ value: Dim) -> Self { set("minWidth", value.ir) }
    @discardableResult public func minHeight(_ value: Dim) -> Self { set("minHeight", value.ir) }
    @discardableResult public func maxWidth(_ value: Dim) -> Self { set("maxWidth", value.ir) }
    @discardableResult public func maxHeight(_ value: Dim) -> Self { set("maxHeight", value.ir) }

    /// CSS 1–4 value shorthand. An omitted side follows CSS — right defaults to
    /// top, bottom to top, left to right — so `padding(top: 20)` behaves.
    @discardableResult
    public func padding(_ top: Dim, _ right: Dim? = nil, _ bottom: Dim? = nil, _ left: Dim? = nil) -> Self {
        box(["padding", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft"], top, right, bottom, left)
    }

    @discardableResult
    public func padding(top: Dim? = nil, right: Dim? = nil, bottom: Dim? = nil, left: Dim? = nil) -> Self {
        sides(["paddingTop", "paddingRight", "paddingBottom", "paddingLeft"], top, right, bottom, left)
    }

    @discardableResult
    public func margin(_ top: Dim, _ right: Dim? = nil, _ bottom: Dim? = nil, _ left: Dim? = nil) -> Self {
        box(["margin", "marginTop", "marginRight", "marginBottom", "marginLeft"], top, right, bottom, left)
    }

    @discardableResult
    public func margin(top: Dim? = nil, right: Dim? = nil, bottom: Dim? = nil, left: Dim? = nil) -> Self {
        sides(["marginTop", "marginRight", "marginBottom", "marginLeft"], top, right, bottom, left)
    }

    @discardableResult
    public func borderWidth(_ top: Dim, _ right: Dim? = nil, _ bottom: Dim? = nil, _ left: Dim? = nil) -> Self {
        box(["borderWidth", "borderTopWidth", "borderRightWidth", "borderBottomWidth", "borderLeftWidth"],
            top, right, bottom, left)
    }

    @discardableResult
    public func borderWidth(top: Dim? = nil, right: Dim? = nil, bottom: Dim? = nil, left: Dim? = nil) -> Self {
        sides(["borderTopWidth", "borderRightWidth", "borderBottomWidth", "borderLeftWidth"],
              top, right, bottom, left)
    }

    @discardableResult public func borderColor(_ value: String) -> Self { set("borderColor", .string(value)) }
    @discardableResult public func top(_ value: Dim) -> Self { set("top", value.ir) }
    @discardableResult public func right(_ value: Dim) -> Self { set("right", value.ir) }
    @discardableResult public func bottom(_ value: Dim) -> Self { set("bottom", value.ir) }
    @discardableResult public func left(_ value: Dim) -> Self { set("left", value.ir) }

    /// The leading inset, which flips with the writing direction.
    @discardableResult public func start(_ value: Dim) -> Self { set("start", value.ir) }

    /// The trailing inset, which flips with the writing direction.
    @discardableResult public func end(_ value: Dim) -> Self { set("end", value.ir) }
    @discardableResult public func inset(_ value: Dim) -> Self { set("inset", value.ir) }

    @discardableResult public func gridColumn(_ start: Int, span: Int? = nil) -> Self {
        props.set("gridColumnStart", .integer(start))
        if let span { props.set("gridColumnSpan", .integer(span)) }
        return self
    }

    @discardableResult public func gridRow(_ start: Int, span: Int? = nil) -> Self {
        props.set("gridRowStart", .integer(start))
        if let span { props.set("gridRowSpan", .integer(span)) }
        return self
    }

    /// Force or forbid a page break at this node. Needs `pageHeight`.
    @discardableResult public func pageBreak(_ value: PageBreakMode) -> Self { set("pageBreak", .string(value.rawValue)) }

    @discardableResult public func translateX(_ value: Double) -> Self { set("translateX", .number(value)) }
    @discardableResult public func translateY(_ value: Double) -> Self { set("translateY", .number(value)) }

    /// Rotation in degrees, about the node's centre.
    @discardableResult public func rotate(_ degrees: Double) -> Self { set("rotation", .number(degrees)) }

    /// Scale. One argument scales both axes.
    @discardableResult public func scale(_ x: Double, _ y: Double? = nil) -> Self {
        set("scale", .array([.number(x), .number(y ?? x)]))
    }

    /// Add background layers: CSS colours, gradients, or a `Photo`.
    @discardableResult public func bg(_ layers: Paint...) -> Self {
        props.push("background", layers.map { $0.ir })
        return self
    }

    @discardableResult public func background(_ layers: Paint...) -> Self {
        props.push("background", layers.map { $0.ir })
        return self
    }

    @discardableResult public func opacity(_ value: Double) -> Self { set("opacity", .number(value)) }

    /// Corner radii: one value for all four, or up to four from the top left.
    @discardableResult public func cornerRadius(_ radii: Double...) -> Self {
        set("cornerRadius", .array(radii.map { .number($0) }))
    }

    @discardableResult public func rounded(_ radii: Double...) -> Self {
        set("cornerRadius", .array(radii.map { .number($0) }))
    }

    @discardableResult public func borderRadius(_ radii: Double...) -> Self {
        set("cornerRadius", .array(radii.map { .number($0) }))
    }

    /// Squircle-ness, 0..1. Figma's corner smoothing.
    @discardableResult public func cornerSmoothing(_ value: Double) -> Self { set("cornerSmoothing", .number(value)) }
    @discardableResult public func corner(_ value: Corner) -> Self { set("corner", .string(value.rawValue)) }

    /// Add CSS `box-shadow` strings.
    @discardableResult public func shadow(_ shadows: String...) -> Self {
        props.push("shadows", shadows.map { .string($0) })
        return self
    }

    // CSS filters, applied in the order they are added.
    @discardableResult public func blur(_ radius: Double) -> Self { filter("blur(\(Num.of(radius))px)") }
    @discardableResult public func brightness(_ amount: Double) -> Self { filter("brightness(\(Num.of(amount)))") }
    @discardableResult public func contrast(_ amount: Double) -> Self { filter("contrast(\(Num.of(amount)))") }
    @discardableResult public func grayscale(_ amount: Double) -> Self { filter("grayscale(\(Num.of(amount)))") }
    @discardableResult public func hueRotate(_ degrees: Double) -> Self { filter("hue-rotate(\(Num.of(degrees)))") }
    @discardableResult public func invert(_ amount: Double) -> Self { filter("invert(\(Num.of(amount)))") }
    @discardableResult public func saturate(_ amount: Double) -> Self { filter("saturate(\(Num.of(amount)))") }
    @discardableResult public func sepia(_ amount: Double) -> Self { filter("sepia(\(Num.of(amount)))") }

    @discardableResult public func filter(_ css: String) -> Self {
        props.push("filters", [.string(css)])
        return self
    }

    private func box(_ keys: [String], _ top: Dim, _ right: Dim?, _ bottom: Dim?, _ left: Dim?) -> Self {
        if right == nil && bottom == nil && left == nil {
            props.set(keys[0], top.ir)
            return self
        }
        props.set(keys[1], top.ir)
        props.set(keys[2], (right ?? top).ir)
        props.set(keys[3], (bottom ?? top).ir)
        props.set(keys[4], (left ?? right ?? top).ir)
        return self
    }

    private func sides(_ keys: [String], _ top: Dim?, _ right: Dim?, _ bottom: Dim?, _ left: Dim?) -> Self {
        guard let anchor = top ?? right ?? bottom ?? left else { return self }
        props.set(keys[0], (top ?? anchor).ir)
        props.set(keys[1], (right ?? top ?? anchor).ir)
        props.set(keys[2], (bottom ?? top ?? anchor).ir)
        props.set(keys[3], (left ?? right ?? top ?? anchor).ir)
        return self
    }
}

extension SpanStyled {
    @discardableResult public func color(_ value: String) -> Self { set("color", .string(value)) }

    /// The font size, not the box size.
    @discardableResult public func size(_ value: Double) -> Self { set("size", .number(value)) }

    /// The font stack, in fallback order.
    @discardableResult public func font(_ families: String...) -> Self {
        set("font", .array(families.map { .string($0) }))
    }

    @discardableResult public func style(_ value: FontStyle) -> Self { set("style", .string(value.rawValue)) }
    @discardableResult public func weight(_ value: Weight) -> Self { set("weight", value.ir) }
    @discardableResult public func letterSpacing(_ value: Double) -> Self { set("letterSpacing", .number(value)) }
    @discardableResult public func wordSpacing(_ value: Double) -> Self { set("wordSpacing", .number(value)) }

    @discardableResult public func underline(_ thickness: Double = 1.0) -> Self { set("underline", .number(thickness)) }
    @discardableResult public func overline(_ thickness: Double = 1.0) -> Self { set("overline", .number(thickness)) }
    @discardableResult public func lineThrough(_ thickness: Double = 1.0) -> Self { set("lineThrough", .number(thickness)) }

    /// Passing nil is an explicit null: "use the text colour".
    @discardableResult public func underlineColor(_ value: String? = nil) -> Self {
        props.setNullable("underlineColor", value.map { .string($0) })
        return self
    }

    @discardableResult public func overlineColor(_ value: String? = nil) -> Self {
        props.setNullable("overlineColor", value.map { .string($0) })
        return self
    }

    @discardableResult public func lineThroughColor(_ value: String? = nil) -> Self {
        props.setNullable("lineThroughColor", value.map { .string($0) })
        return self
    }

    @discardableResult public func highlight(_ value: String? = nil) -> Self {
        props.setNullable("highlightColor", value.map { .string($0) })
        return self
    }

    /// Add CSS `text-shadow` strings.
    @discardableResult public func dropShadow(_ shadows: String...) -> Self {
        props.push("dropShadows", shadows.map { .string($0) })
        return self
    }

    /// The glyph outline colour.
    @discardableResult public func strokeColor(_ value: String) -> Self { set("strokeColor", .string(value)) }

    /// The glyph outline width.
    @discardableResult public func strokeWidth(_ value: Double) -> Self { set("strokeWidth", .number(value)) }

    /// Shift the run off its baseline — superscripts, subscripts.
    @discardableResult public func offsetY(_ value: Double) -> Self { set("offsetY", .number(value)) }

    /// Force this run's direction, overriding bidi resolution.
    @discardableResult public func textDir(_ value: WritingDirection) -> Self { set("textDir", .string(value.rawValue)) }
}

extension TextBlock {
    @discardableResult public func nowrap(_ value: Bool = true) -> Self { set("nowrap", .boolean(value)) }

    /// Whether the paragraph wraps. Not the flexbox `wrap`.
    @discardableResult public func wrapText(_ value: Bool = true) -> Self { set("nowrap", .boolean(!value)) }

    @discardableResult public func maxLines(_ value: Double) -> Self { set("maxLines", .number(value)) }
    @discardableResult public func lineBreak(_ value: LineBreakMode) -> Self { set("lineBreak", .string(value.rawValue)) }
    @discardableResult public func textOverflow(_ value: TextOverflow) -> Self { set("textOverflow", .string(value.rawValue)) }
    @discardableResult public func lineHeight(_ value: Double) -> Self { set("lineHeight", .number(value)) }
    @discardableResult public func align(_ value: TextAlign) -> Self { set("align", .string(value.rawValue)) }
    @discardableResult public func indent(_ value: Double) -> Self { set("indentSize", .number(value)) }
    @discardableResult public func hangingIndent(_ value: Double) -> Self { set("hangingIndentSize", .number(value)) }
    @discardableResult public func tabStops(_ stops: Double...) -> Self { set("tabStops", .array(stops.map { .number($0) })) }
    @discardableResult public func tabLeader(_ value: String) -> Self { set("tabLeader", .string(value)) }
    @discardableResult public func autofit(_ value: Bool = true) -> Self { set("autofit", .boolean(value)) }

    /// Rotation of the text inside its box, in degrees.
    @discardableResult public func orientation(_ degrees: Int) -> Self { set("orientation", .integer(degrees)) }

    /// Paint the glyphs with an image instead of a colour.
    @discardableResult public func clipImage(_ photo: Photo) -> Self { set("clipImage", .node(photo)) }

    /// The base direction used to resolve bidi runs.
    @discardableResult public func baseDir(_ value: BaseDirection) -> Self { set("baseDir", .string(value.rawValue)) }

    /// Greedy wrapping, or balancing for a ragged edge.
    @discardableResult public func textWrap(_ value: TextWrapMode) -> Self { set("textWrap", .string(value.rawValue)) }
}

extension Node {
    @discardableResult
    func set(_ key: String, _ value: IRValue) -> Self {
        props.set(key, value)
        return self
    }
}
