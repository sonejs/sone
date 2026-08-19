// Keyword values. Dart has no union types, so every keyword the IR accepts is
// an enum here, and `apply` is the escape hatch for anything not yet modelled.

/// How wrapped flex lines are distributed on the cross axis.
enum AlignContent {
  flexStart('flex-start'),
  flexEnd('flex-end'),
  center('center'),
  stretch('stretch'),
  spaceBetween('space-between'),
  spaceAround('space-around'),
  spaceEvenly('space-evenly');

  const AlignContent(this.value);

  final String value;
}

/// Cross-axis alignment. Also the type of `alignSelf`.
enum AlignItems {
  flexStart('flex-start'),
  flexEnd('flex-end'),
  center('center'),
  stretch('stretch'),
  baseline('baseline');

  const AlignItems(this.value);

  final String value;
}

/// Main-axis distribution.
enum JustifyContent {
  flexStart('flex-start'),
  flexEnd('flex-end'),
  center('center'),
  spaceBetween('space-between'),
  spaceAround('space-around'),
  spaceEvenly('space-evenly');

  const JustifyContent(this.value);

  final String value;
}

/// The main axis of a container.
enum FlexDirection {
  row('row'),
  column('column'),
  rowReverse('row-reverse'),
  columnReverse('column-reverse');

  const FlexDirection(this.value);

  final String value;
}

/// Whether flex items wrap onto new lines.
enum FlexWrap {
  wrap('wrap'),
  noWrap('nowrap'),
  wrapReverse('wrap-reverse');

  const FlexWrap(this.value);

  final String value;
}

/// Whether width and height include padding and border.
enum BoxSizing {
  borderBox('border-box'),
  contentBox('content-box');

  const BoxSizing(this.value);

  final String value;
}

/// Writing direction.
enum TextDirection {
  ltr('ltr'),
  rtl('rtl');

  const TextDirection(this.value);

  final String value;
}

/// How a node participates in layout.
enum Display {
  none('none'),
  flex('flex'),
  contents('contents');

  const Display(this.value);

  final String value;
}

/// What happens to content past a node's box.
enum Overflow {
  visible('visible'),
  hidden('hidden'),
  scroll('scroll');

  const Overflow(this.value);

  final String value;
}

/// Positioning scheme.
enum Position {
  absolute('absolute'),
  relative('relative'),
  static('static');

  const Position(this.value);

  final String value;
}

/// Where a page break may or must fall.
enum PageBreakMode {
  before('before'),
  after('after'),
  avoid('avoid');

  const PageBreakMode(this.value);

  final String value;
}

/// The shape a corner radius produces.
enum Corner {
  cut('cut'),
  round('round');

  const Corner(this.value);

  final String value;
}

/// How a photo fills its box.
enum ScaleType {
  cover('cover'),
  fill('fill'),
  contain('contain');

  const ScaleType(this.value);

  final String value;
}

/// Roman or slanted.
enum FontStyle {
  normal('normal'),
  italic('italic'),
  oblique('oblique');

  const FontStyle(this.value);

  final String value;
}

/// The line-breaking algorithm.
enum LineBreakMode {
  greedy('greedy'),
  knuthPlass('knuth-plass');

  const LineBreakMode(this.value);

  final String value;
}

/// What a clipped paragraph ends with.
enum TextOverflow {
  clip('clip'),
  ellipsis('ellipsis');

  const TextOverflow(this.value);

  final String value;
}

/// Horizontal alignment inside a paragraph.
enum TextAlign {
  left('left'),
  right('right'),
  center('center'),
  justify('justify');

  const TextAlign(this.value);

  final String value;
}

/// Greedy wrapping, or ragged-edge balancing.
enum TextWrapMode {
  wrap('wrap'),
  balance('balance');

  const TextWrapMode(this.value);

  final String value;
}

/// How an open path ends.
enum StrokeCap {
  butt('butt'),
  round('round'),
  square('square');

  const StrokeCap(this.value);

  final String value;
}

/// How two path segments meet.
enum StrokeJoin {
  bevel('bevel'),
  miter('miter'),
  round('round');

  const StrokeJoin(this.value);

  final String value;
}

/// Which regions of a self-intersecting path are inside it.
enum FillRule {
  evenOdd('evenodd'),
  nonZero('nonzero');

  const FillRule(this.value);

  final String value;
}

/// The paragraph's base direction for bidi resolution.
enum BaseDirection {
  ltr('ltr'),
  rtl('rtl'),
  auto('auto');

  const BaseDirection(this.value);

  final String value;
}

/// Whether the final page is full height or shrinks to its content.
enum LastPageHeight {
  uniform('uniform'),
  content('content');

  const LastPageHeight(this.value);

  final String value;
}

/// The granularity of the boxes `metadata` returns.
enum Granularity {
  node('node'),
  line('line'),
  word('word');

  const Granularity(this.value);

  final String value;
}
