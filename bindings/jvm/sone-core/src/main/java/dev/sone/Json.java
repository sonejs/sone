package dev.sone;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * Just enough JSON for the IR.
 *
 * <p>Writing is all this binding needs to render; {@code layoutJson()} and
 * {@code metadataJson()} hand back the engine's JSON as a string rather than a
 * parsed tree, so nobody inherits a Jackson or Gson version from us. The only
 * reading here is the one array the C ABI returns.
 */
public final class Json {

    private Json() {
    }

    // An instanceof chain rather than a pattern-matching switch: this module
    // compiles at Java 17 so it dexes cleanly for Android, and that syntax is 21.
    static void write(StringBuilder out, Object value) {
        if (value == null) {
            out.append("null");
        } else if (value instanceof String text) {
            string(out, text);
        } else if (value instanceof Boolean flag) {
            out.append(flag);
        } else if (value instanceof Node node) {
            node.writeJson(out);
        } else if (value instanceof Dim dim) {
            write(out, dim.toIr());
        } else if (value instanceof Track track) {
            write(out, track.toIr());
        } else if (value instanceof Number number) {
            number(out, number);
        } else if (value instanceof Map<?, ?> map) {
            out.append('{');
            boolean first = true;
            for (Map.Entry<?, ?> entry : map.entrySet()) {
                if (!first) {
                    out.append(',');
                }
                first = false;
                string(out, String.valueOf(entry.getKey()));
                out.append(':');
                write(out, entry.getValue());
            }
            out.append('}');
        } else if (value instanceof Iterable<?> items) {
            out.append('[');
            boolean first = true;
            for (Object item : items) {
                if (!first) {
                    out.append(',');
                }
                first = false;
                write(out, item);
            }
            out.append(']');
        } else {
            throw new IllegalArgumentException(
                    value.getClass() + " cannot be written to the IR");
        }
    }

    private static void number(StringBuilder out, Number number) {
        double value = number.doubleValue();
        if (value == Math.rint(value) && !Double.isInfinite(value)) {
            out.append((long) value);
        } else {
            out.append(value);
        }
    }

    private static void string(StringBuilder out, String text) {
        out.append('"');
        for (int i = 0; i < text.length(); i++) {
            char c = text.charAt(i);
            switch (c) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                case '\b' -> out.append("\\b");
                case '\f' -> out.append("\\f");
                default -> {
                    // Non-ASCII goes through as UTF-8: the engine reads it
                    // directly, so escaping would only make the document bigger.
                    if (c < 0x20) {
                        out.append(String.format("\\u%04x", (int) c));
                    } else {
                        out.append(c);
                    }
                }
            }
        }
        out.append('"');
    }

    /** Reads the one shape the C ABI returns: a flat array of strings. */
    public static List<String> readStringArray(String json) {
        List<String> out = new ArrayList<>();
        int index = json.indexOf('[');
        if (index < 0) {
            return out;
        }
        StringBuilder current = null;
        boolean escaped = false;
        for (int i = index + 1; i < json.length(); i++) {
            char c = json.charAt(i);
            if (current == null) {
                if (c == '"') {
                    current = new StringBuilder();
                } else if (c == ']') {
                    break;
                }
                continue;
            }
            if (escaped) {
                current.append(switch (c) {
                    case 'n' -> '\n';
                    case 't' -> '\t';
                    case 'r' -> '\r';
                    default -> c;
                });
                escaped = false;
            } else if (c == '\\') {
                escaped = true;
            } else if (c == '"') {
                out.add(current.toString());
                current = null;
            } else {
                current.append(c);
            }
        }
        return out;
    }
}
