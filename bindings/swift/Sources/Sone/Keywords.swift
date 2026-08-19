// Keyword values are enums with String raw values, so a call site reads as
// `.spaceBetween` with no type to name and no string to misspell.

/// How wrapped flex lines are distributed on the cross axis.
public enum AlignContent: String {
    case flexStart = "flex-start"
    case flexEnd = "flex-end"
    case center = "center"
    case stretch = "stretch"
    case spaceBetween = "space-between"
    case spaceAround = "space-around"
    case spaceEvenly = "space-evenly"
}

/// Cross-axis alignment. Also the type of `alignSelf`.
public enum AlignItems: String {
    case flexStart = "flex-start"
    case flexEnd = "flex-end"
    case center = "center"
    case stretch = "stretch"
    case baseline = "baseline"
}

/// Main-axis distribution.
public enum JustifyContent: String {
    case flexStart = "flex-start"
    case flexEnd = "flex-end"
    case center = "center"
    case spaceBetween = "space-between"
    case spaceAround = "space-around"
    case spaceEvenly = "space-evenly"
}

/// The main axis of a container.
public enum FlexDirection: String {
    case row = "row"
    case column = "column"
    case rowReverse = "row-reverse"
    case columnReverse = "column-reverse"
}

/// Whether flex items wrap onto new lines.
public enum FlexWrap: String {
    case wrap = "wrap"
    case nowrap = "nowrap"
    case wrapReverse = "wrap-reverse"
}

/// Whether width and height include padding and border.
public enum BoxSizing: String {
    case borderBox = "border-box"
    case contentBox = "content-box"
}

/// Writing direction.
public enum WritingDirection: String {
    case ltr = "ltr"
    case rtl = "rtl"
}

/// How a node participates in layout.
public enum Display: String {
    case none = "none"
    case flex = "flex"
    case contents = "contents"
}

/// What happens to content past a node's box.
public enum Overflow: String {
    case visible = "visible"
    case hidden = "hidden"
    case scroll = "scroll"
}

/// Positioning scheme.
public enum Position: String {
    case absolute = "absolute"
    case relative = "relative"
    case `static` = "static"
}

/// Where a page break may or must fall.
public enum PageBreakMode: String {
    case before = "before"
    case after = "after"
    case avoid = "avoid"
}

/// The shape a corner radius produces.
public enum Corner: String {
    case cut = "cut"
    case round = "round"
}

/// How a photo fills its box.
public enum ScaleType: String {
    case cover = "cover"
    case fill = "fill"
    case contain = "contain"
}

/// Roman or slanted.
public enum FontStyle: String {
    case normal = "normal"
    case italic = "italic"
    case oblique = "oblique"
}

/// The line-breaking algorithm.
public enum LineBreakMode: String {
    case greedy = "greedy"
    case knuthPlass = "knuth-plass"
}

/// What a clipped paragraph ends with.
public enum TextOverflow: String {
    case clip = "clip"
    case ellipsis = "ellipsis"
}

/// Horizontal alignment inside a paragraph.
public enum TextAlign: String {
    case left = "left"
    case right = "right"
    case center = "center"
    case justify = "justify"
}

/// Greedy wrapping, or ragged-edge balancing.
public enum TextWrapMode: String {
    case wrap = "wrap"
    case balance = "balance"
}

/// How an open path ends.
public enum StrokeCap: String {
    case butt = "butt"
    case round = "round"
    case square = "square"
}

/// How two path segments meet.
public enum StrokeJoin: String {
    case bevel = "bevel"
    case miter = "miter"
    case round = "round"
}

/// Which regions of a self-intersecting path are inside it.
public enum FillRule: String {
    case evenOdd = "evenodd"
    case nonZero = "nonzero"
}

/// The paragraph's base direction for bidi resolution.
public enum BaseDirection: String {
    case ltr = "ltr"
    case rtl = "rtl"
    case auto = "auto"
}

/// Whether the final page is full height or shrinks to its content.
public enum LastPageHeight: String {
    case uniform = "uniform"
    case content = "content"
}

/// The granularity of the boxes `metadata` returns.
public enum Granularity: String {
    case node = "node"
    case line = "line"
    case word = "word"
}

/// The output formats the engine can encode.
public enum OutputFormat: String {
    case png = "png"
    case jpeg = "jpeg"
    case webp = "webp"
    case raw = "raw"
    case pdf = "pdf"
    case svg = "svg"

    /// The `SoneFormat` discriminant the C ABI expects.
    var code: Int32 {
        switch self {
        case .png: return 0
        case .jpeg: return 1
        case .webp: return 2
        case .raw: return 3
        case .pdf: return 4
        case .svg: return 5
        }
    }
}
