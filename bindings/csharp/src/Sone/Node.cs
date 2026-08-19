using System.Text.Json;

namespace Sone;

/// <summary>
/// A node in the document tree. Build one with the factories on <see cref="Dsl"/>.
/// </summary>
public interface INode
{
    /// <summary>The IR node type, e.g. <c>"column"</c>.</summary>
    string Type { get; }

    /// <summary>The properties set on this node, in the order they were set.</summary>
    PropBag Props { get; }

    /// <summary>Container children. Empty for <c>text</c> and <c>span</c>.</summary>
    IList<INode> Children { get; }

    /// <summary>Paragraph content. Only <c>text</c> and <c>span</c> use it.</summary>
    IList<Inline> InlineContent { get; }
}

/// <summary>
/// Marker for nodes that carry flexbox, sizing, spacing and paint properties.
/// The properties themselves are extension methods, so a node can be layout,
/// span and paragraph at once without multiple inheritance.
/// </summary>
public interface ILayoutNode : INode
{
}

/// <summary>Marker for nodes that carry span-level text styling.</summary>
public interface ISpanStyle : INode
{
}

/// <summary>Marker for nodes that carry paragraph-level properties.</summary>
public interface ITextBlock : INode
{
}

/// <summary>
/// A node's properties. Insertion-ordered so the serialized IR is stable, which
/// matters for golden tests more than lookup speed does — a node has a handful
/// of properties, never hundreds.
/// </summary>
public sealed class PropBag
{
    private readonly List<KeyValuePair<string, object?>> _entries = [];

    public int Count => _entries.Count;

    /// <summary>Set a property, ignoring nulls the way an unset argument should be.</summary>
    public void Set(string key, object? value)
    {
        if (value is not null)
        {
            Assign(key, value);
        }
    }

    /// <summary>
    /// Set a property that may legitimately be null. An explicit null clears a
    /// decoration colour, which the engine distinguishes from the property
    /// being absent.
    /// </summary>
    public void SetNullable(string key, object? value) => Assign(key, value);

    /// <summary>Append to a list-valued property such as <c>background</c> or <c>filters</c>.</summary>
    public void Push(string key, IEnumerable<object?> values)
    {
        var index = IndexOf(key);
        if (index < 0)
        {
            _entries.Add(new KeyValuePair<string, object?>(key, new List<object?>(values)));
            return;
        }
        if (_entries[index].Value is not List<object?> list)
        {
            throw new InvalidOperationException($"property \"{key}\" is not a list");
        }
        list.AddRange(values);
    }

    public bool TryGetValue(string key, out object? value)
    {
        var index = IndexOf(key);
        value = index < 0 ? null : _entries[index].Value;
        return index >= 0;
    }

    private void Assign(string key, object? value)
    {
        var index = IndexOf(key);
        var entry = new KeyValuePair<string, object?>(key, value);
        if (index < 0)
        {
            _entries.Add(entry);
        }
        else
        {
            _entries[index] = entry;
        }
    }

    private int IndexOf(string key)
    {
        for (var i = 0; i < _entries.Count; i++)
        {
            if (_entries[i].Key == key)
            {
                return i;
            }
        }
        return -1;
    }

    internal void WriteTo(Utf8JsonWriter writer)
    {
        writer.WriteStartObject();
        foreach (var (key, value) in _entries)
        {
            writer.WritePropertyName(key);
            Ir.WriteValue(writer, value);
        }
        writer.WriteEndObject();
    }
}

/// <summary>
/// The base for every node. Concrete types add only the properties that are
/// theirs alone; everything shared lives in the extension methods.
/// </summary>
public abstract class Node : INode
{
    protected Node(string type)
    {
        Type = type;
    }

    public string Type { get; }

    public PropBag Props { get; } = new();

    public IList<INode> Children { get; } = new List<INode>();

    public IList<Inline> InlineContent { get; } = new List<Inline>();

    public override string ToString()
    {
        var tag = Props.TryGetValue("tag", out var value) ? $" \"{value}\"" : "";
        return $"<{Type}{tag} props={Props.Count} children={Children.Count}>";
    }

    internal void Adopt(IEnumerable<INode?> children)
    {
        foreach (var child in children)
        {
            if (child is not null)
            {
                Children.Add(child);
            }
        }
    }
}

/// <summary>Serializes a tree to the IR wire format.</summary>
internal static class Ir
{
    internal const int Version = 1;

    internal static void WriteNode(Utf8JsonWriter writer, INode node)
    {
        writer.WriteStartObject();
        writer.WriteString("type", node.Type);
        if (node.Props.Count > 0)
        {
            writer.WritePropertyName("props");
            node.Props.WriteTo(writer);
        }
        if (node.Children.Count > 0)
        {
            writer.WritePropertyName("children");
            writer.WriteStartArray();
            foreach (var child in node.Children)
            {
                WriteNode(writer, child);
            }
            writer.WriteEndArray();
        }
        if (node.InlineContent.Count > 0)
        {
            writer.WritePropertyName("inline");
            writer.WriteStartArray();
            foreach (var inline in node.InlineContent)
            {
                WriteValue(writer, inline.ToIr());
            }
            writer.WriteEndArray();
        }
        writer.WriteEndObject();
    }

    internal static void WriteValue(Utf8JsonWriter writer, object? value)
    {
        switch (value)
        {
            case null:
                writer.WriteNullValue();
                break;
            case string text:
                writer.WriteStringValue(text);
                break;
            case bool flag:
                writer.WriteBooleanValue(flag);
                break;
            case double number:
                writer.WriteNumberValue(number);
                break;
            case int number:
                writer.WriteNumberValue(number);
                break;
            case INode node:
                WriteNode(writer, node);
                break;
            case IReadOnlyList<object?> list:
                writer.WriteStartArray();
                foreach (var item in list)
                {
                    WriteValue(writer, item);
                }
                writer.WriteEndArray();
                break;
            case IReadOnlyDictionary<string, object?> map:
                writer.WriteStartObject();
                foreach (var (key, item) in map)
                {
                    writer.WritePropertyName(key);
                    WriteValue(writer, item);
                }
                writer.WriteEndObject();
                break;
            default:
                throw new InvalidOperationException(
                    $"{value.GetType()} cannot be written to the IR — convert it before setting the property");
        }
    }
}
