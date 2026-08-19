import Foundation

/// A value as the IR carries it.
public enum IRValue {
    case string(String)
    case number(Double)
    case integer(Int)
    case boolean(Bool)
    case array([IRValue])
    case object([(String, IRValue)])
    case node(Node)
    case null
}

extension IRValue: ExpressibleByStringLiteral {
    public init(stringLiteral value: String) { self = .string(value) }
}

extension IRValue: ExpressibleByFloatLiteral {
    public init(floatLiteral value: Double) { self = .number(value) }
}

extension IRValue: ExpressibleByIntegerLiteral {
    public init(integerLiteral value: Int) { self = .integer(value) }
}

/// A length: a number, `auto`, or a percentage.
///
/// The literal conformances are what make the IR's `number | "auto" | "%"`
/// union one method: `.width(100)`, `.width("50%")` and `.width(.auto)` all
/// call the same thing.
public enum Dim {
    case points(Double)
    case percent(Double)
    case auto

    var ir: IRValue {
        switch self {
        case .points(let value): return .number(value)
        case .percent(let value): return .string("\(Num.of(value))%")
        case .auto: return .string("auto")
        }
    }
}

extension Dim: ExpressibleByIntegerLiteral {
    public init(integerLiteral value: Int) { self = .points(Double(value)) }
}

extension Dim: ExpressibleByFloatLiteral {
    public init(floatLiteral value: Double) { self = .points(value) }
}

extension Dim: ExpressibleByStringLiteral {
    public init(stringLiteral value: String) {
        let trimmed = value.trimmingCharacters(in: .whitespaces)
        if trimmed == "auto" {
            self = .auto
        } else if trimmed.hasSuffix("%"), let number = Double(trimmed.dropLast()) {
            self = .percent(number)
        } else if let number = Double(trimmed) {
            self = .points(number)
        } else {
            preconditionFailure("invalid length \"\(value)\" — expected a number, \"auto\", or a percentage")
        }
    }
}

/// A grid track: a fixed size, `auto`, or an `fr` share.
public enum Track {
    case points(Double)
    case fr(Double)
    case auto

    var ir: IRValue {
        switch self {
        case .points(let value): return .number(value)
        case .fr(let value): return .string("\(Num.of(value))fr")
        case .auto: return .string("auto")
        }
    }
}

extension Track: ExpressibleByIntegerLiteral {
    public init(integerLiteral value: Int) { self = .points(Double(value)) }
}

extension Track: ExpressibleByFloatLiteral {
    public init(floatLiteral value: Double) { self = .points(value) }
}

/// A font weight: a CSS keyword or a number.
public enum Weight {
    case keyword(String)
    case value(Double)

    public static let normal = Weight.keyword("normal")
    public static let bold = Weight.keyword("bold")
    public static let lighter = Weight.keyword("lighter")
    public static let bolder = Weight.keyword("bolder")

    var ir: IRValue {
        switch self {
        case .keyword(let text): return .string(text)
        case .value(let number): return .number(number)
        }
    }
}

extension Weight: ExpressibleByIntegerLiteral {
    public init(integerLiteral value: Int) { self = .value(Double(value)) }
}

extension Weight: ExpressibleByStringLiteral {
    public init(stringLiteral value: String) { self = .keyword(value) }
}

/// A background layer: a CSS colour or gradient, or a `Photo`.
public enum Paint {
    case css(String)
    case photo(Photo)

    var ir: IRValue {
        switch self {
        case .css(let text): return .string(text)
        case .photo(let node): return .node(node)
        }
    }
}

extension Paint: ExpressibleByStringLiteral {
    public init(stringLiteral value: String) { self = .css(value) }
}

/// Page margins. A single number applies to all four sides.
public struct Margin {
    public var top: Double
    public var right: Double
    public var bottom: Double
    public var left: Double

    public init(top: Double = 0, right: Double = 0, bottom: Double = 0, left: Double = 0) {
        self.top = top
        self.right = right
        self.bottom = bottom
        self.left = left
    }

    public init(_ all: Double) {
        self.init(top: all, right: all, bottom: all, left: all)
    }

    var ir: IRValue {
        .object([("top", .number(top)), ("right", .number(right)),
                 ("bottom", .number(bottom)), ("left", .number(left))])
    }
}

extension Margin: ExpressibleByIntegerLiteral {
    public init(integerLiteral value: Int) { self.init(Double(value)) }
}

extension Margin: ExpressibleByFloatLiteral {
    public init(floatLiteral value: Double) { self.init(value) }
}

/// A font the document carries with it, so another sone engine renders it identically.
public struct FontSource {
    public let name: String
    public let src: String

    public init(_ name: String, _ src: String) {
        self.name = name
        self.src = src
    }
}

/// Formats a Double the way CSS wants it: no trailing `.0`.
enum Num {
    static func of(_ value: Double) -> String {
        value == value.rounded() && value.isFinite
            ? String(Int(value))
            : String(value)
    }
}
