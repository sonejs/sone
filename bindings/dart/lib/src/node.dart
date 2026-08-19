import 'dart:convert';

import 'keywords.dart';

/// A length: a number, `'auto'`, or a percentage such as `'50%'`.
///
/// Dart has no union types, so this is `Object` with a check at the call site
/// rather than a wrapper you would have to construct.
typedef Dim = Object;

/// A grid track: a number, `'auto'`, or an `fr` share such as `'1fr'`.
typedef Track = Object;

Object _dim(Object value, String property) {
  if (value is num) return value;
  if (value is String) {
    if (value == 'auto' || RegExp(r'^-?[\d.]+%?$').hasMatch(value))
      return value;
  }
  throw ArgumentError.value(
      value, property, 'expected a number, "auto", or a percentage');
}

/// A node in the document tree.
///
/// Setters return `void` and are meant to be used with cascades — `..` already
/// evaluates to the receiver, so none of the self-type machinery the JVM and
/// .NET bindings need is required here.
abstract class Node {
  Node(this.type, {List<Node>? children})
      : children = List<Node>.from(children ?? const <Node>[]);

  /// The IR node type, e.g. `'column'`.
  final String type;

  /// Properties set on this node, in the order they were set.
  final Map<String, Object?> props = <String, Object?>{};

  /// Container children. Empty for `text` and `span`.
  final List<Node> children;

  /// Paragraph content: `String`s and [Span]s. Only `text` and `span` use it.
  final List<Object> inline = <Object>[];

  /// A name for this node, echoed back by `layout()` and `metadata()`.
  void tag(String value) => set('tag', value);

  /// Set raw IR properties, for anything this API does not cover yet.
  void apply(Map<String, Object?> values) => props.addAll(values);

  /// Set a property, ignoring nulls the way an omitted argument should be.
  void set(String key, Object? value) {
    if (value != null) props[key] = value;
  }

  /// Set a property that may legitimately be null — an explicit null clears a
  /// decoration colour, which the engine reads differently from unset.
  void setNullable(String key, Object? value) => props[key] = value;

  /// Append to a list-valued property such as `background` or `filters`.
  void push(String key, Iterable<Object?> values) {
    final list = props.putIfAbsent(key, () => <Object?>[]) as List<Object?>;
    list.addAll(values);
  }

  /// This node as an IR document fragment.
  Map<String, Object?> toIr() => <String, Object?>{
        'type': type,
        if (props.isNotEmpty) 'props': props,
        if (children.isNotEmpty)
          'children': children.map((child) => child.toIr()).toList(),
        if (inline.isNotEmpty)
          'inline':
              inline.map((item) => item is Node ? item.toIr() : item).toList(),
      };

  /// This node as IR JSON.
  String toJsonString() => jsonEncode(toIr());

  @override
  String toString() {
    final label = props['tag'] != null ? " '${props['tag']}'" : '';
    return '<sone:$type$label props=${props.length} children=${children.length}>';
  }
}

/// Flexbox, sizing, spacing and the visual box properties.
mixin LayoutProps on Node {
  void alignContent(AlignContent value) => set('alignContent', value.value);
  void alignItems(AlignItems value) => set('alignItems', value.value);
  void alignSelf(AlignItems value) => set('alignSelf', value.value);
  void aspectRatio(num value) => set('aspectRatio', value);
  void boxSizing(BoxSizing value) => set('boxSizing', value.value);
  void direction(TextDirection value) => set('direction', value.value);
  void display(Display value) => set('display', value.value);
  void flex(num value) => set('flex', value);
  void basis(Dim value) => set('flexBasis', _dim(value, 'basis'));
  void flexDirection(FlexDirection value) => set('flexDirection', value.value);
  void grow(num value) => set('flexGrow', value);
  void shrink(num value) => set('flexShrink', value);
  void wrap(FlexWrap value) => set('flexWrap', value.value);
  void justifyContent(JustifyContent value) =>
      set('justifyContent', value.value);
  void overflow(Overflow value) => set('overflow', value.value);
  void position(Position value) => set('position', value.value);

  void gap(num value) => set('gap', value);
  void rowGap(num value) => set('rowGap', value);
  void columnGap(num value) => set('columnGap', value);

  /// Width and height. One argument makes a square.
  void size(Dim width, [Dim? height]) {
    set('width', _dim(width, 'size'));
    set('height', _dim(height ?? width, 'size'));
  }

  void width(Dim value) => set('width', _dim(value, 'width'));
  void height(Dim value) => set('height', _dim(value, 'height'));
  void minWidth(Dim value) => set('minWidth', _dim(value, 'minWidth'));
  void minHeight(Dim value) => set('minHeight', _dim(value, 'minHeight'));
  void maxWidth(Dim value) => set('maxWidth', _dim(value, 'maxWidth'));
  void maxHeight(Dim value) => set('maxHeight', _dim(value, 'maxHeight'));

  /// CSS 1-4 value shorthand. An omitted side follows CSS: right defaults to
  /// top, bottom to top, left to right.
  void padding(Dim top, [Dim? right, Dim? bottom, Dim? left]) => _box(const [
        'padding',
        'paddingTop',
        'paddingRight',
        'paddingBottom',
        'paddingLeft'
      ], top, right, bottom, left);

  void margin(Dim top, [Dim? right, Dim? bottom, Dim? left]) => _box(const [
        'margin',
        'marginTop',
        'marginRight',
        'marginBottom',
        'marginLeft'
      ], top, right, bottom, left);

  void borderWidth(num top, [num? right, num? bottom, num? left]) =>
      _box(const [
        'borderWidth',
        'borderTopWidth',
        'borderRightWidth',
        'borderBottomWidth',
        'borderLeftWidth'
      ], top, right, bottom, left);

  void borderColor(String value) => set('borderColor', value);

  void marginTop(Dim value) => set('marginTop', _dim(value, 'marginTop'));
  void marginRight(Dim value) => set('marginRight', _dim(value, 'marginRight'));
  void marginBottom(Dim value) =>
      set('marginBottom', _dim(value, 'marginBottom'));
  void marginLeft(Dim value) => set('marginLeft', _dim(value, 'marginLeft'));
  void paddingTop(Dim value) => set('paddingTop', _dim(value, 'paddingTop'));
  void paddingRight(Dim value) =>
      set('paddingRight', _dim(value, 'paddingRight'));
  void paddingBottom(Dim value) =>
      set('paddingBottom', _dim(value, 'paddingBottom'));
  void paddingLeft(Dim value) => set('paddingLeft', _dim(value, 'paddingLeft'));

  void top(Dim value) => set('top', _dim(value, 'top'));
  void right(Dim value) => set('right', _dim(value, 'right'));
  void bottom(Dim value) => set('bottom', _dim(value, 'bottom'));
  void left(Dim value) => set('left', _dim(value, 'left'));

  /// The leading inset, which flips with the writing direction.
  void start(Dim value) => set('start', _dim(value, 'start'));

  /// The trailing inset, which flips with the writing direction.
  void end(Dim value) => set('end', _dim(value, 'end'));
  void inset(Dim value) => set('inset', _dim(value, 'inset'));

  void gridColumn(int start, [int? span]) {
    set('gridColumnStart', start);
    set('gridColumnSpan', span);
  }

  void gridRow(int start, [int? span]) {
    set('gridRowStart', start);
    set('gridRowSpan', span);
  }

  /// Force or forbid a page break at this node. Needs `pageHeight`.
  void pageBreak(PageBreakMode value) => set('pageBreak', value.value);

  void translateX(num value) => set('translateX', value);
  void translateY(num value) => set('translateY', value);

  /// Rotation in degrees, about the node's centre.
  void rotate(num degrees) => set('rotation', degrees);

  /// Scale. One argument scales both axes.
  void scale(num x, [num? y]) => set('scale', <num>[x, y ?? x]);

  /// Add background layers: CSS colours, gradients, or a [Photo].
  void bg(Object layer, [Object? second, Object? third]) => background(
      <Object>[layer, if (second != null) second, if (third != null) third]);

  void background(Iterable<Object> layers) => push('background',
      layers.map((layer) => layer is Node ? layer.toIr() : layer));

  void opacity(num value) => set('opacity', value);

  /// Corner radii: one value for all four, or up to four clockwise from the
  /// top left.
  void cornerRadius(num first, [num? second, num? third, num? fourth]) =>
      set('cornerRadius', <num>[
        first,
        if (second != null) second,
        if (third != null) third,
        if (fourth != null) fourth,
      ]);

  void rounded(num first, [num? second, num? third, num? fourth]) =>
      cornerRadius(first, second, third, fourth);

  /// Corner radii from a list, for the named-argument form where a variadic
  /// call site is not available.
  void cornerRadii(Iterable<num> radii) => set('cornerRadius', radii.toList());

  void borderRadius(num first, [num? second, num? third, num? fourth]) =>
      cornerRadius(first, second, third, fourth);

  /// Squircle-ness, 0..1. Figma's corner smoothing.
  void cornerSmoothing(num value) => set('cornerSmoothing', value);
  void corner(Corner value) => set('corner', value.value);

  /// Add CSS `box-shadow` strings.
  void shadow(String value) => push('shadows', <String>[value]);

  // CSS filters, applied in the order they are added.
  void blur(num radius) => _filter('blur(${_n(radius)}px)');
  void brightness(num amount) => _filter('brightness(${_n(amount)})');
  void contrast(num amount) => _filter('contrast(${_n(amount)})');
  void grayscale(num amount) => _filter('grayscale(${_n(amount)})');
  void hueRotate(num degrees) => _filter('hue-rotate(${_n(degrees)})');
  void invert(num amount) => _filter('invert(${_n(amount)})');
  void saturate(num amount) => _filter('saturate(${_n(amount)})');
  void sepia(num amount) => _filter('sepia(${_n(amount)})');

  void _filter(String css) => push('filters', <String>[css]);

  static String _n(num value) => value is int || value == value.roundToDouble()
      ? value.toInt().toString()
      : value.toString();

  void _box(List<String> keys, Object top, Object? right, Object? bottom,
      Object? left) {
    if (right == null && bottom == null && left == null) {
      set(keys[0], _dim(top, keys[0]));
      return;
    }
    set(keys[1], _dim(top, keys[1]));
    set(keys[2], _dim(right ?? top, keys[2]));
    set(keys[3], _dim(bottom ?? top, keys[3]));
    set(keys[4], _dim(left ?? right ?? top, keys[4]));
  }
}

/// Span-level text styling.
mixin SpanStyleProps on Node {
  void color(String value) => set('color', value);

  /// The font size. Exposed as `size` on the nodes that carry it — see
  /// [Text.size] — but named apart here because [LayoutProps.size] is the box.
  void fontSize(num value) => set('size', value);

  /// The font stack, in fallback order.
  void font(String family, [String? fallback, String? lastResort]) =>
      set('font', <String>[
        family,
        if (fallback != null) fallback,
        if (lastResort != null) lastResort,
      ]);

  void style(FontStyle value) => set('style', value.value);

  /// A CSS keyword such as `'bold'`, or a number.
  void weight(Object value) => set('weight', value);
  void letterSpacing(num value) => set('letterSpacing', value);
  void wordSpacing(num value) => set('wordSpacing', value);

  void underline([num thickness = 1.0]) => set('underline', thickness);

  /// Pass nothing for an explicit null, which means "use the text colour".
  void underlineColor([String? value]) => setNullable('underlineColor', value);
  void overline([num thickness = 1.0]) => set('overline', thickness);
  void overlineColor([String? value]) => setNullable('overlineColor', value);
  void lineThrough([num thickness = 1.0]) => set('lineThrough', thickness);
  void lineThroughColor([String? value]) =>
      setNullable('lineThroughColor', value);
  void highlight([String? value]) => setNullable('highlightColor', value);

  /// Add CSS `text-shadow` strings.
  void dropShadow(String value) => push('dropShadows', <String>[value]);

  /// The glyph outline colour.
  void strokeColor(String value) => set('strokeColor', value);

  /// The glyph outline width.
  void strokeWidth(num value) => set('strokeWidth', value);

  /// Shift the run off its baseline — superscripts, subscripts.
  void offsetY(num value) => set('offsetY', value);

  /// Force this run's direction, overriding bidi resolution.
  void textDir(TextDirection value) => set('textDir', value.value);
}

/// Paragraph-level properties.
mixin TextBlockProps on Node {
  void nowrap([bool value = true]) => set('nowrap', value);

  /// Whether the paragraph wraps. Not the flexbox `wrap`.
  void wrapText([bool value = true]) => set('nowrap', !value);

  void maxLines(num value) => set('maxLines', value);
  void lineBreak(LineBreakMode value) => set('lineBreak', value.value);
  void textOverflow(TextOverflow value) => set('textOverflow', value.value);
  void lineHeight(num value) => set('lineHeight', value);
  void align(TextAlign value) => set('align', value.value);
  void indent(num value) => set('indentSize', value);
  void hangingIndent(num value) => set('hangingIndentSize', value);
  void tabStops(Iterable<num> stops) => set('tabStops', stops.toList());
  void tabLeader(String value) => set('tabLeader', value);
  void autofit([bool value = true]) => set('autofit', value);

  /// Rotation of the text inside its box, in degrees.
  void orientation(int degrees) => set('orientation', degrees);

  /// Paint the glyphs with an image instead of a colour.
  void clipImage(Node photo) => set('clipImage', photo.toIr());

  /// The base direction used to resolve bidi runs.
  void baseDir(BaseDirection value) => set('baseDir', value.value);

  /// Greedy wrapping, or balancing for a ragged edge.
  void textWrap(TextWrapMode value) => set('textWrap', value.value);
}
