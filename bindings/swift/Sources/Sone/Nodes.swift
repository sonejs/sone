import Foundation

/// Collects children written as statements in a trailing closure — the same
/// shape SwiftUI uses, and what gives `if` and `for` inside the block.
@resultBuilder
public enum NodeBuilder {
    public static func buildBlock(_ components: [Node]...) -> [Node] { components.flatMap { $0 } }
    public static func buildExpression(_ expression: Node) -> [Node] { [expression] }
    public static func buildExpression(_ expression: [Node]) -> [Node] { expression }
    public static func buildOptional(_ component: [Node]?) -> [Node] { component ?? [] }
    public static func buildEither(first component: [Node]) -> [Node] { component }
    public static func buildEither(second component: [Node]) -> [Node] { component }
    public static func buildArray(_ components: [[Node]]) -> [Node] { components.flatMap { $0 } }
    public static func buildLimitedAvailability(_ component: [Node]) -> [Node] { component }
}

/// Collects paragraph content.
@resultBuilder
public enum InlineBuilder {
    public static func buildBlock(_ components: [Inline]...) -> [Inline] { components.flatMap { $0 } }
    public static func buildExpression(_ expression: Inline) -> [Inline] { [expression] }
    public static func buildExpression(_ expression: String) -> [Inline] { [.text(expression)] }
    public static func buildExpression(_ expression: Span) -> [Inline] { [.span(expression)] }
    public static func buildExpression(_ expression: [Inline]) -> [Inline] { expression }
    public static func buildOptional(_ component: [Inline]?) -> [Inline] { component ?? [] }
    public static func buildEither(first component: [Inline]) -> [Inline] { component }
    public static func buildEither(second component: [Inline]) -> [Inline] { component }
    public static func buildArray(_ components: [[Inline]]) -> [Inline] { components.flatMap { $0 } }
}

/// A vertical container.
public final class Column: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "column")
        self.children = children()
    }
}

/// A horizontal container.
public final class Row: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "row")
        self.children = children()
    }
}

/// A grid container with row-major auto placement.
public final class Grid: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "grid")
        self.children = children()
    }

    @discardableResult public func columns(_ tracks: Track...) -> Self { set("columns", .array(tracks.map { $0.ir })) }
    @discardableResult public func rows(_ tracks: Track...) -> Self { set("rows", .array(tracks.map { $0.ir })) }
    @discardableResult public func autoRows(_ tracks: Track...) -> Self { set("autoRows", .array(tracks.map { $0.ir })) }
    @discardableResult public func autoColumns(_ tracks: Track...) -> Self { set("autoColumns", .array(tracks.map { $0.ir })) }
}

/// A styled run inside a `Text`.
public final class Span: Node, SpanStyled {
    public init(_ text: String) {
        super.init(type: "span")
        inline = [.text(text)]
    }
}

/// A paragraph. Both a box and a run of text — Swift protocols compose all three
/// property sets with no single-inheritance fight.
public final class Text: Node, LayoutNode, SpanStyled, TextBlock {
    public init(_ text: String) {
        super.init(type: "text")
        inline = [.text(text)]
    }

    public init(@InlineBuilder _ content: () -> [Inline]) {
        super.init(type: "text")
        inline = content()
    }
}

/// Cascades text styling onto its descendants without drawing a box.
public final class TextDefault: Node, SpanStyled, TextBlock {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "text-default")
        self.children = children()
    }
}

/// An image.
public final class Photo: Node, LayoutNode {
    /// From a path, a URL, or `asset:name`.
    public init(_ src: String) {
        super.init(type: "photo")
        props.set("src", .string(src))
    }

    /// From raw bytes, inlined into the document as a data URL.
    public init(data: Data) {
        super.init(type: "photo")
        props.set("src", .string("data:application/octet-stream;base64," + data.base64EncodedString()))
    }

    /// How the image fills its box. The alignment is 0..1.
    @discardableResult public func scaleType(_ value: ScaleType, alignment: Double? = nil) -> Self {
        props.set("scaleType", .string(value.rawValue))
        if let alignment { props.set("scaleAlignment", .number(alignment)) }
        return self
    }

    @discardableResult public func preserveAspectRatio(_ value: Bool = true) -> Self { set("preserveAspectRatio", .boolean(value)) }
    @discardableResult public func flipHorizontal(_ value: Bool = true) -> Self { set("flipHorizontal", .boolean(value)) }
    @discardableResult public func flipVertical(_ value: Bool = true) -> Self { set("flipVertical", .boolean(value)) }

    /// The letterbox colour behind a `contain` image.
    @discardableResult public func fill(_ color: String) -> Self { set("fill", .string(color)) }

    /// An SVG path the image is clipped to.
    @discardableResult public func clipPath(_ path: String) -> Self { set("clipPath", .string(path)) }
}

/// An SVG path.
public final class SvgPath: Node, LayoutNode {
    public init(_ d: String) {
        super.init(type: "path")
        props.set("d", .string(d))
    }

    @discardableResult public func stroke(_ color: String) -> Self { set("stroke", .string(color)) }
    @discardableResult public func strokeWidth(_ value: Double) -> Self { set("strokeWidth", .number(value)) }
    @discardableResult public func strokeLineCap(_ value: StrokeCap) -> Self { set("strokeLineCap", .string(value.rawValue)) }
    @discardableResult public func strokeLineJoin(_ value: StrokeJoin) -> Self { set("strokeLineJoin", .string(value.rawValue)) }
    @discardableResult public func strokeMiterLimit(_ value: Double) -> Self { set("strokeMiterLimit", .number(value)) }
    @discardableResult public func strokeDashArray(_ values: Double...) -> Self { set("strokeDashArray", .array(values.map { .number($0) })) }
    @discardableResult public func strokeDashOffset(_ value: Double) -> Self { set("strokeDashOffset", .number(value)) }
    @discardableResult public func fill(_ color: String) -> Self { set("fill", .string(color)) }
    @discardableResult public func fillOpacity(_ value: Double) -> Self { set("fillOpacity", .number(value)) }
    @discardableResult public func fillRule(_ value: FillRule) -> Self { set("fillRule", .string(value.rawValue)) }

    /// Scale the path data itself, before layout.
    @discardableResult public func scalePath(_ value: Double) -> Self { set("scalePath", .number(value)) }
}

/// A table. Children are `TableRow`s.
public final class Table: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "table")
        self.children = children()
    }

    /// Row and column spacing. One argument sets both.
    @discardableResult public func spacing(_ row: Double, _ column: Double? = nil) -> Self {
        set("spacing", .array([.number(row), .number(column ?? row)]))
    }
}

/// A table row. Children are `TableCell`s.
public final class TableRow: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "table-row")
        self.children = children()
    }
}

/// A table cell.
public final class TableCell: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "table-cell")
        self.children = children()
    }

    @discardableResult public func colspan(_ value: Int) -> Self { set("colspan", .integer(value)) }
    @discardableResult public func rowspan(_ value: Int) -> Self { set("rowspan", .integer(value)) }
}

/// A bulleted or numbered list. Named `Bullets` because `List` is SwiftUI's.
public final class Bullets: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "list")
        self.children = children()
    }

    /// `disc`, `circle`, `square`, `decimal`, `dash`, `none`, or literal text.
    @discardableResult public func listStyle(_ value: String) -> Self { set("listStyle", .string(value)) }

    /// A styled marker node. `{}` in its text is replaced with the item number.
    @discardableResult public func listStyle(_ marker: Node) -> Self { set("listStyle", .node(marker)) }

    @discardableResult public func markerGap(_ value: Double) -> Self { set("markerGap", .number(value)) }
    @discardableResult public func markerOffset(_ value: Double) -> Self { set("markerOffset", .number(value)) }
    @discardableResult public func startIndex(_ value: Int) -> Self { set("startIndex", .integer(value)) }
}

/// One item in a `Bullets` list.
public final class ListItem: Node, LayoutNode {
    public init(@NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "list-item")
        self.children = children()
    }

    /// Override the list's marker for this item alone.
    @discardableResult public func marker(_ value: Node) -> Self { set("marker", .node(value)) }
}

/// Clips every child to an SVG path.
public final class ClipGroup: Node, LayoutNode {
    public init(_ clipPath: String, @NodeBuilder _ children: () -> [Node] = { [] }) {
        super.init(type: "clip-group")
        props.set("clipPath", .string(clipPath))
        self.children = children()
    }
}

/// An explicit page break. Only meaningful with `pageHeight` set.
public func PageBreak() -> Column {
    Column().height(0).pageBreak(.before)
}
