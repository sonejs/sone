#!/usr/bin/env python3
"""Generates lib/src/nodes.dart.

The named-argument constructors and the cascade setters have to stay in step,
so the parameter lists are derived from one table rather than typed twice.

    python3 tool/generate_nodes.py
"""
import pathlib

# (param name, Dart type, apply statement). `{n}` is the parameter name.
LAYOUT = [
    ("tag", "String", "tag({n})"),
    ("alignContent", "AlignContent", "alignContent({n})"),
    ("alignItems", "AlignItems", "alignItems({n})"),
    ("alignSelf", "AlignItems", "alignSelf({n})"),
    ("aspectRatio", "num", "aspectRatio({n})"),
    ("boxSizing", "BoxSizing", "boxSizing({n})"),
    ("direction", "TextDirection", "direction({n})"),
    ("display", "Display", "display({n})"),
    ("flex", "num", "flex({n})"),
    ("basis", "Dim", "basis({n})"),
    ("flexDirection", "FlexDirection", "flexDirection({n})"),
    ("grow", "num", "grow({n})"),
    ("shrink", "num", "shrink({n})"),
    ("wrap", "FlexWrap", "wrap({n})"),
    ("justifyContent", "JustifyContent", "justifyContent({n})"),
    ("overflow", "Overflow", "overflow({n})"),
    ("position", "Position", "position({n})"),
    ("gap", "num", "gap({n})"),
    ("rowGap", "num", "rowGap({n})"),
    ("columnGap", "num", "columnGap({n})"),
    ("size", "Dim", "size({n})"),
    ("width", "Dim", "width({n})"),
    ("height", "Dim", "height({n})"),
    ("minWidth", "Dim", "minWidth({n})"),
    ("minHeight", "Dim", "minHeight({n})"),
    ("maxWidth", "Dim", "maxWidth({n})"),
    ("maxHeight", "Dim", "maxHeight({n})"),
    ("padding", "Dim", "padding({n})"),
    ("paddingTop", "Dim", "paddingTop({n})"),
    ("paddingRight", "Dim", "paddingRight({n})"),
    ("paddingBottom", "Dim", "paddingBottom({n})"),
    ("paddingLeft", "Dim", "paddingLeft({n})"),
    ("margin", "Dim", "margin({n})"),
    ("marginTop", "Dim", "marginTop({n})"),
    ("marginRight", "Dim", "marginRight({n})"),
    ("marginBottom", "Dim", "marginBottom({n})"),
    ("marginLeft", "Dim", "marginLeft({n})"),
    ("borderWidth", "num", "borderWidth({n})"),
    ("borderColor", "String", "borderColor({n})"),
    ("top", "Dim", "top({n})"),
    ("right", "Dim", "right({n})"),
    ("bottom", "Dim", "bottom({n})"),
    ("left", "Dim", "left({n})"),
    ("start", "Dim", "start({n})"),
    ("end", "Dim", "end({n})"),
    ("inset", "Dim", "inset({n})"),
    ("gridColumn", "int", "gridColumn({n}, gridColumnSpan)"),
    ("gridColumnSpan", "int", None),  # consumed by gridColumn
    ("gridRow", "int", "gridRow({n}, gridRowSpan)"),
    ("gridRowSpan", "int", None),
    ("pageBreak", "PageBreakMode", "pageBreak({n})"),
    ("translateX", "num", "translateX({n})"),
    ("translateY", "num", "translateY({n})"),
    ("rotate", "num", "rotate({n})"),
    ("scale", "num", "scale({n})"),
    ("opacity", "num", "opacity({n})"),
    ("bg", "Object", "bg({n})"),
    ("backgrounds", "List<Object>", "background({n})"),
    ("cornerRadius", "num", "cornerRadius({n})"),
    ("cornerRadii", "List<num>", "cornerRadii({n})"),
    ("cornerSmoothing", "num", "cornerSmoothing({n})"),
    ("corner", "Corner", "corner({n})"),
    ("shadows", "List<String>", "for (final value in {n}) shadow(value)"),
    # Filters are applied in the order below. Cascades are the way to control
    # that order yourself.
    ("blur", "num", "blur({n})"),
    ("brightness", "num", "brightness({n})"),
    ("contrast", "num", "contrast({n})"),
    ("grayscale", "num", "grayscale({n})"),
    ("hueRotate", "num", "hueRotate({n})"),
    ("invert", "num", "invert({n})"),
    ("saturate", "num", "saturate({n})"),
    ("sepia", "num", "sepia({n})"),
]

SPAN = [
    ("color", "String", "color({n})"),
    ("fontSize", "num", "fontSize({n})"),
    ("font", "Object", "set('font', {n} is Iterable ? {n}.toList() : [{n}])"),
    ("style", "FontStyle", "style({n})"),
    ("weight", "Object", "weight({n})"),
    ("letterSpacing", "num", "letterSpacing({n})"),
    ("wordSpacing", "num", "wordSpacing({n})"),
    ("underline", "num", "underline({n})"),
    ("underlineColor", "String", "underlineColor({n})"),
    ("overline", "num", "overline({n})"),
    ("overlineColor", "String", "overlineColor({n})"),
    ("lineThrough", "num", "lineThrough({n})"),
    ("lineThroughColor", "String", "lineThroughColor({n})"),
    ("highlight", "String", "highlight({n})"),
    ("dropShadows", "List<String>", "for (final value in {n}) dropShadow(value)"),
    ("strokeColor", "String", "strokeColor({n})"),
    ("strokeWidth", "num", "strokeWidth({n})"),
    ("offsetY", "num", "offsetY({n})"),
    ("textDir", "TextDirection", "textDir({n})"),
]

TEXT_BLOCK = [
    ("nowrap", "bool", "nowrap({n})"),
    ("wrapText", "bool", "wrapText({n})"),
    ("maxLines", "num", "maxLines({n})"),
    ("lineBreak", "LineBreakMode", "lineBreak({n})"),
    ("textOverflow", "TextOverflow", "textOverflow({n})"),
    ("lineHeight", "num", "lineHeight({n})"),
    ("align", "TextAlign", "align({n})"),
    ("indent", "num", "indent({n})"),
    ("hangingIndent", "num", "hangingIndent({n})"),
    ("tabStops", "List<num>", "tabStops({n})"),
    ("tabLeader", "String", "tabLeader({n})"),
    ("autofit", "bool", "autofit({n})"),
    ("orientation", "int", "orientation({n})"),
    ("clipImage", "Node", "clipImage({n})"),
    ("baseDir", "BaseDirection", "baseDir({n})"),
    ("textWrap", "TextWrapMode", "textWrap({n})"),
]

GRID = [
    ("columns", "List<Track>", "columns({n})"),
    ("rows", "List<Track>", "rows({n})"),
    ("autoRows", "List<Track>", "autoRows({n})"),
    ("autoColumns", "List<Track>", "autoColumns({n})"),
]

PHOTO = [
    ("scaleType", "ScaleType", "scaleType({n}, scaleAlignment)"),
    ("scaleAlignment", "Object", None),
    ("preserveAspectRatio", "bool", "preserveAspectRatio({n})"),
    ("flipHorizontal", "bool", "flipHorizontal({n})"),
    ("flipVertical", "bool", "flipVertical({n})"),
    ("fill", "String", "fill({n})"),
    ("clipPath", "String", "clipPath({n})"),
]

SVG_PATH = [
    ("stroke", "String", "stroke({n})"),
    ("strokeWidth", "num", "strokeWidth({n})"),
    ("strokeLineCap", "StrokeCap", "strokeLineCap({n})"),
    ("strokeLineJoin", "StrokeJoin", "strokeLineJoin({n})"),
    ("strokeMiterLimit", "num", "strokeMiterLimit({n})"),
    ("strokeDashArray", "List<num>", "strokeDashArray({n})"),
    ("strokeDashOffset", "num", "strokeDashOffset({n})"),
    ("fill", "String", "fill({n})"),
    ("fillOpacity", "num", "fillOpacity({n})"),
    ("fillRule", "FillRule", "fillRule({n})"),
    ("scalePath", "num", "scalePath({n})"),
]

TABLE = [
    ("spacing", "num", "spacing({n}, spacingColumn)"),
    ("spacingColumn", "num", None),
]

TABLE_CELL = [
    ("colspan", "int", "colspan({n})"),
    ("rowspan", "int", "rowspan({n})"),
]

BULLETS = [
    ("listStyle", "String", "listStyle({n})"),
    ("listStyleNode", "Node", "listStyleNode({n})"),
    ("markerGap", "num", "markerGap({n})"),
    ("markerOffset", "num", "markerOffset({n})"),
    ("startIndex", "int", "startIndex({n})"),
]

LIST_ITEM = [("marker", "Node", "marker({n})")]



# The node-specific cascade setters. The named arguments above call straight
# into these, so the two forms can never drift.
GRID_METHODS = """
  void columns(Iterable<Track> tracks) => set('columns', _tracks(tracks));
  void rows(Iterable<Track> tracks) => set('rows', _tracks(tracks));
  void autoRows(Iterable<Track> tracks) => set('autoRows', _tracks(tracks));
  void autoColumns(Iterable<Track> tracks) => set('autoColumns', _tracks(tracks));

  static List<Object> _tracks(Iterable<Track> tracks) => tracks.map((track) {
        if (track is num) return track;
        if (track is String &&
            (track == 'auto' || RegExp(r'^[\\d.]+fr$').hasMatch(track))) {
          return track;
        }
        throw ArgumentError.value(
            track, 'track', 'expected a number, "auto", or an fr value');
      }).toList();"""

PHOTO_METHODS = """
  static const _alignments = <String, double>{
    'start': 0.0,
    'center': 0.5,
    'end': 1.0,
  };

  /// How the image fills its box. The alignment is 0..1, or one of `'start'`,
  /// `'center'`, `'end'`.
  void scaleType(ScaleType value, [Object? alignment]) {
    set('scaleType', value.value);
    if (alignment == null) return;
    set(
      'scaleAlignment',
      alignment is String
          ? (_alignments[alignment] ??
              (throw ArgumentError.value(alignment, 'alignment')))
          : alignment,
    );
  }

  void preserveAspectRatio([bool value = true]) =>
      set('preserveAspectRatio', value);
  void flipHorizontal([bool value = true]) => set('flipHorizontal', value);
  void flipVertical([bool value = true]) => set('flipVertical', value);

  /// The letterbox colour behind a `contain` image.
  void fill(String color) => set('fill', color);

  /// An SVG path the image is clipped to.
  void clipPath(String path) => set('clipPath', path);"""

SVG_PATH_METHODS = """
  void stroke(String color) => set('stroke', color);
  void strokeWidth(num value) => set('strokeWidth', value);
  void strokeLineCap(StrokeCap value) => set('strokeLineCap', value.value);
  void strokeLineJoin(StrokeJoin value) => set('strokeLineJoin', value.value);
  void strokeMiterLimit(num value) => set('strokeMiterLimit', value);
  void strokeDashArray(Iterable<num> values) =>
      set('strokeDashArray', values.toList());
  void strokeDashOffset(num value) => set('strokeDashOffset', value);
  void fill(String color) => set('fill', color);
  void fillOpacity(num value) => set('fillOpacity', value);
  void fillRule(FillRule value) => set('fillRule', value.value);

  /// Scale the path data itself, before layout.
  void scalePath(num value) => set('scalePath', value);"""

TABLE_METHODS = """
  /// Row and column spacing. One argument sets both.
  void spacing(num row, [num? column]) =>
      set('spacing', <num>[row, column ?? row]);"""

TABLE_CELL_METHODS = """
  void colspan(int value) => set('colspan', value);
  void rowspan(int value) => set('rowspan', value);"""

BULLETS_METHODS = """
  /// `disc`, `circle`, `square`, `decimal`, `dash`, `none`, or literal text.
  void listStyle(String value) => set('listStyle', value);

  /// A styled marker node. `{}` in its text is replaced with the item number.
  void listStyleNode(Node marker) => set('listStyle', marker.toIr());

  void markerGap(num value) => set('markerGap', value);
  void markerOffset(num value) => set('markerOffset', value);
  void startIndex(int value) => set('startIndex', value);"""

LIST_ITEM_METHODS = """
  /// Override the list's marker for this item alone.
  void marker(Node value) => set('marker', value.toIr());"""


def params(table, indent="    "):
    return "".join(f"{indent}{kind}? {name},\n" for name, kind, _ in table)


def applies(table, indent="    "):
    out = []
    for name, _, apply in table:
        if apply is None:
            continue
        body = apply.format(n=name)
        if body.startswith("for "):
            out.append(f"{indent}if ({name} != null) {{ {body}; }}")
        else:
            out.append(f"{indent}if ({name} != null) this.{body};")
    return "\n".join(out)


def klass(name, ir_type, tables, *, doc, children=True, positional=None,
          extra_methods="", mixins="LayoutProps"):
    table = [row for group in tables for row in group]
    lines = [f"/// {doc}", f"class {name} extends Node with {mixins} {{"]
    head = f"  {name}("
    if positional:
        head += f"{positional[0]} {positional[1]}, "
    head += "{"
    lines.append(head)
    if children:
        lines.append("    List<Node>? children,")
    lines.append(params(table))
    lines.append(f"  }}) : super('{ir_type}'"
                 + (", children: children" if children else "") + ") {")
    if positional:
        lines.append(positional[2])
    body = applies(table)
    if body:
        lines.append(body)
    lines.append("  }")
    if extra_methods:
        lines.append(extra_methods)
    lines.append("}")
    lines.append("")
    return "\n".join(lines)


header = '''// GENERATED by tool/generate_nodes.py — do not edit by hand.
//
// Every property is a named constructor argument, the shape a Flutter reader
// expects. The cascade setters in node.dart are unchanged and still work, and
// they remain the way to do the two things named arguments cannot express:
// control the order filters are applied in, and pass an explicit null to a
// decoration colour (a null argument here means "unset", not "use the text
// colour").

import 'dart:convert';

import 'keywords.dart';
import 'node.dart';

'''

out = [header]

out.append(klass("Column", "column", [LAYOUT], doc="A vertical container."))
out.append(klass("Row", "row", [LAYOUT], doc="A horizontal container."))
out.append(klass("Grid", "grid", [LAYOUT, GRID],
                 doc="A grid container with row-major auto placement.", extra_methods=GRID_METHODS))
out.append(klass("TableRow", "table-row", [LAYOUT], doc="A table row. Children are [TableCell]s."))
out.append(klass("TableCell", "table-cell", [LAYOUT, TABLE_CELL], doc="A table cell.", extra_methods=TABLE_CELL_METHODS))
out.append(klass("ListItem", "list-item", [LAYOUT, LIST_ITEM], doc="One item in a [Bullets] list.", extra_methods=LIST_ITEM_METHODS))
out.append(klass("Table", "table", [LAYOUT, TABLE], doc="A table. Children are [TableRow]s.", extra_methods=TABLE_METHODS))
out.append(klass("Bullets", "list", [LAYOUT, BULLETS],
                 doc="A bulleted or numbered list. Named `Bullets` because `List` is `dart:core`'s.", extra_methods=BULLETS_METHODS))

out.append(klass(
    "Photo", "photo", [LAYOUT, PHOTO],
    doc="An image. [src] is a path, a URL, `asset:name`, or raw bytes.",
    children=False,
    positional=("Object", "src", """    set('src', src is String
        ? src
        : 'data:application/octet-stream;base64,${base64Encode((src as List).cast<int>())}');"""),
    extra_methods=PHOTO_METHODS,
))

out.append(klass(
    "SvgPath", "path", [LAYOUT, SVG_PATH],
    doc="An SVG path. Named `SvgPath` because `Path` is `dart:ui`'s in Flutter.",
    children=False,
    positional=("String", "d", "    set('d', d);"),
    extra_methods=SVG_PATH_METHODS,
))

out.append(klass(
    "ClipGroup", "clip-group", [LAYOUT],
    doc="Clips every child to an SVG path.",
    positional=("String", "clipPath", "    set('clipPath', clipPath);"),
))

out.append(klass(
    "Span", "span", [SPAN],
    doc="A styled run inside a [Text].",
    children=False,
    mixins="SpanStyleProps",
    positional=("String", "text", "    inline.add(text);"),
    extra_methods="""
  /// The font size. A span has no box, so there is nothing to disambiguate.
  void size(num value) => fontSize(value);

  /// Append raw text.
  void content(String text) => inline.add(text);""",
))

out.append(klass(
    "TextDefault", "text-default", [SPAN, TEXT_BLOCK],
    doc="Cascades text styling onto its descendants without drawing a box.",
    mixins="SpanStyleProps, TextBlockProps",
    extra_methods="""
  /// The font size cascaded onto descendants.
  void size(num value) => fontSize(value);""",
))

# Text carries all three property sets. `size` means the font size here, so the
# layout `size` is dropped from the named arguments — use width and height.
TEXT_LAYOUT = [row for row in LAYOUT if row[0] != "size"]
TEXT_OWN = [("size", "num", "size({n})")]
out.append(klass(
    "Text", "text", [TEXT_LAYOUT, TEXT_OWN, SPAN, TEXT_BLOCK],
    doc="""A paragraph. [content] is a String, or a list of Strings and [Span]s.
///
/// `size` is the **font** size here, not the box size — matching the TypeScript
/// API, where `TextPropsBuilder` omits the layout `size`. Use `width` and
/// `height` for the box.""",
    children=False,
    mixins="LayoutProps, SpanStyleProps, TextBlockProps",
    positional=("Object?", "content", """    if (content is String) {
      inline.add(content);
    } else if (content is Iterable) {
      for (final item in content) {
        inline.add(item is Span ? item : item as String);
      }
    } else if (content != null) {
      throw ArgumentError.value(content, 'content', 'expected a String or a list');
    }"""),
    extra_methods="""
  /// The **font** size, not the box size. Both included mixins would otherwise
  /// resolve `size` to the layout one, and the `size:` named argument calls
  /// straight through here — so this override is what keeps the two honest.
  @override
  void size(Object value, [Object? height]) {
    if (height != null) {
      throw ArgumentError.value(height, 'height',
          'Text.size is the font size; use width and height for the box');
    }
    if (value is! num) {
      throw ArgumentError.value(value, 'size', 'expected a font size in points');
    }
    fontSize(value);
  }

  /// Append raw text.
  void content(String text) => inline.add(text);

  /// Append a styled run.
  void span(Span run) => inline.add(run);""",
))

out.append("""/// An explicit page break. Only meaningful with `pageHeight` set.
Column pageBreak() => Column(height: 0, pageBreak: PageBreakMode.before);
""")

pathlib.Path("lib/src/nodes.dart").write_text("\n".join(out))
print("wrote lib/src/nodes.dart")
