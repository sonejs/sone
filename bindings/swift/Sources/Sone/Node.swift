import Foundation

/// A node's properties, kept in the order they were set so the serialized IR is
/// stable. A node has a handful of properties, never hundreds, so a linear scan
/// is the right trade.
public struct Props {
    private(set) var entries: [(key: String, value: IRValue)] = []

    public init() {}

    /// Set a property, ignoring nil the way an omitted argument should be.
    public mutating func set(_ key: String, _ value: IRValue?) {
        guard let value else { return }
        assign(key, value)
    }

    /// Set a property that may legitimately be null — an explicit null clears a
    /// decoration colour, which the engine reads differently from unset.
    public mutating func setNullable(_ key: String, _ value: IRValue?) {
        assign(key, value ?? .null)
    }

    /// Append to a list-valued property such as `background` or `filters`.
    public mutating func push(_ key: String, _ values: [IRValue]) {
        if let index = entries.firstIndex(where: { $0.key == key }),
           case .array(let existing) = entries[index].value {
            entries[index].value = .array(existing + values)
        } else {
            assign(key, .array(values))
        }
    }

    public subscript(key: String) -> IRValue? {
        entries.first(where: { $0.key == key })?.value
    }

    private mutating func assign(_ key: String, _ value: IRValue) {
        if let index = entries.firstIndex(where: { $0.key == key }) {
            entries[index].value = value
        } else {
            entries.append((key, value))
        }
    }
}

/// A piece of a paragraph.
public enum Inline {
    case text(String)
    case span(Span)
}

extension Inline: ExpressibleByStringLiteral {
    public init(stringLiteral value: String) { self = .text(value) }
}

/// A node in the document tree.
///
/// A class rather than a struct, so a property method can return `Self` and a
/// chain keeps its concrete type — Swift gives that for free on a
/// class-constrained protocol extension, with none of the self-type machinery
/// the JVM and .NET bindings need.
public class Node {
    public let type: String
    public var props = Props()
    public var children: [Node] = []
    public var inline: [Inline] = []

    init(type: String) {
        self.type = type
    }

    /// A name for this node, echoed back by `layoutJSON()` and `metadataJSON()`.
    @discardableResult
    public func tag(_ value: String) -> Self {
        props.set("tag", .string(value))
        return self
    }

    /// Set raw IR properties, for anything this API does not cover yet.
    @discardableResult
    public func apply(_ values: [(String, IRValue)]) -> Self {
        for (key, value) in values { props.set(key, value) }
        return self
    }

    /// This node as IR JSON.
    public func toJSON() -> String {
        var out = ""
        write(into: &out)
        return out
    }

    func write(into out: inout String) {
        out += "{\"type\":"
        Json.write(.string(type), into: &out)
        if !props.entries.isEmpty {
            out += ",\"props\":"
            Json.write(.object(props.entries.map { ($0.key, $0.value) }), into: &out)
        }
        if !children.isEmpty {
            out += ",\"children\":["
            for (index, child) in children.enumerated() {
                if index > 0 { out += "," }
                child.write(into: &out)
            }
            out += "]"
        }
        if !inline.isEmpty {
            out += ",\"inline\":["
            for (index, item) in inline.enumerated() {
                if index > 0 { out += "," }
                switch item {
                case .text(let text): Json.write(.string(text), into: &out)
                case .span(let span): span.write(into: &out)
                }
            }
            out += "]"
        }
        out += "}"
    }
}

/// Just enough JSON for the IR. Reading is left to the caller's own library:
/// `layoutJSON()` and `metadataJSON()` hand back the engine's JSON as a string,
/// so nobody inherits a serialization dependency from this package.
enum Json {
    static func write(_ value: IRValue, into out: inout String) {
        switch value {
        case .null:
            out += "null"
        case .boolean(let flag):
            out += flag ? "true" : "false"
        case .integer(let number):
            out += String(number)
        case .number(let number):
            out += Num.of(number)
        case .string(let text):
            string(text, into: &out)
        case .node(let node):
            node.write(into: &out)
        case .array(let items):
            out += "["
            for (index, item) in items.enumerated() {
                if index > 0 { out += "," }
                write(item, into: &out)
            }
            out += "]"
        case .object(let pairs):
            out += "{"
            for (index, pair) in pairs.enumerated() {
                if index > 0 { out += "," }
                string(pair.0, into: &out)
                out += ":"
                write(pair.1, into: &out)
            }
            out += "}"
        }
    }

    private static func string(_ text: String, into out: inout String) {
        out += "\""
        for character in text.unicodeScalars {
            switch character {
            case "\"": out += "\\\""
            case "\\": out += "\\\\"
            case "\n": out += "\\n"
            case "\r": out += "\\r"
            case "\t": out += "\\t"
            default:
                // Non-ASCII goes through as UTF-8: the engine reads it directly,
                // so escaping would only make the document bigger.
                if character.value < 0x20 {
                    out += String(format: "\\u%04x", character.value)
                } else {
                    out.unicodeScalars.append(character)
                }
            }
        }
        out += "\""
    }
}
