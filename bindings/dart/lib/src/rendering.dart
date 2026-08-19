import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'engine.dart';
import 'keywords.dart';
import 'native.dart';
import 'node.dart';

/// Page margins. A single number applies to all four sides.
class Margin {
  const Margin({this.top = 0, this.right = 0, this.bottom = 0, this.left = 0});

  const Margin.all(num value)
      : top = value,
        right = value,
        bottom = value,
        left = value;

  final num top;
  final num right;
  final num bottom;
  final num left;

  Map<String, Object?> toIr() =>
      {'top': top, 'right': right, 'bottom': bottom, 'left': left};
}

/// A font the document carries with it, so another sone engine — the CLI, say —
/// renders it identically.
class FontSource {
  const FontSource(this.name, this.src);

  final String name;
  final String src;
}

/// A node plus its render configuration, with one method per output format.
class Rendering {
  Rendering(
    this.root, {
    this.engine,
    this.width,
    this.height,
    this.background,
    this.density,
    this.pageHeight,
    this.margin,
    this.lastPageHeight,
    this.header,
    this.footer,
    this.fonts,
  });

  final Node root;
  final Engine? engine;
  final num? width;
  final num? height;
  final String? background;
  final num? density;
  final num? pageHeight;
  final Margin? margin;
  final LastPageHeight? lastPageHeight;

  /// Drawn at the top of every page. Use the literal tokens `{pageNumber}` and
  /// `{totalPages}` — the engine substitutes them.
  final Node? header;

  /// Drawn at the bottom of every page.
  final Node? footer;

  final List<FontSource>? fonts;

  Engine get _engine => engine ?? Engine.instance;

  String? _cached;

  // ── the document ────────────────────────────────────────────────────────

  /// The IR document as a map.
  Map<String, Object?> toIr() {
    final config = <String, Object?>{
      if (width != null) 'width': width,
      if (height != null) 'height': height,
      if (background != null) 'background': background,
      if (density != null) 'density': density,
      if (pageHeight != null) 'pageHeight': pageHeight,
      if (margin != null) 'margin': margin!.toIr(),
      if (lastPageHeight != null) 'lastPageHeight': lastPageHeight!.value,
      if (header != null) 'header': header!.toIr(),
      if (footer != null) 'footer': footer!.toIr(),
    };
    return <String, Object?>{
      'sone': 1,
      if (fonts != null && fonts!.isNotEmpty)
        'fonts': [
          for (final font in fonts!) {'name': font.name, 'src': font.src},
        ],
      if (config.isNotEmpty) 'config': config,
      'root': root.toIr(),
    };
  }

  /// The IR document as JSON, built once and reused.
  String toJsonString({bool pretty = false}) {
    if (pretty) return const JsonEncoder.withIndent('  ').convert(toIr());
    return _cached ??= jsonEncode(toIr());
  }

  // ── outputs ─────────────────────────────────────────────────────────────

  Uint8List png({double? density}) => _engine.render(toJsonString(),
      format: OutputFormat.png, density: density);

  Uint8List jpeg({double quality = 1.0, double? density}) =>
      _engine.render(toJsonString(),
          format: OutputFormat.jpeg, density: density, quality: quality);

  Uint8List webp({double quality = 1.0, double? density}) =>
      _engine.render(toJsonString(),
          format: OutputFormat.webp, density: density, quality: quality);

  /// Raw RGBA pixels, row-major, unpremultiplied.
  Uint8List raw({double? density}) => _engine.render(toJsonString(),
      format: OutputFormat.raw, density: density);

  /// A PDF. With [pageHeight] set, one page per break and selectable text.
  Uint8List pdf() => _engine.render(toJsonString(), format: OutputFormat.pdf);

  Uint8List svg() => _engine.render(toJsonString(), format: OutputFormat.svg);

  /// One raster image per page. Requires [pageHeight].
  List<Uint8List> pages(
          {OutputFormat format = OutputFormat.png,
          double? density,
          double quality = 1.0}) =>
      _engine.renderPages(toJsonString(),
          format: format, density: density, quality: quality);

  /// Render and write to [path], inferring the format from its extension.
  File save(String path, {double? density, double quality = 1.0}) {
    final bytes = switch (_formatFor(path)) {
      OutputFormat.png => png(density: density),
      OutputFormat.jpeg => jpeg(quality: quality, density: density),
      OutputFormat.webp => webp(quality: quality, density: density),
      OutputFormat.raw => raw(density: density),
      OutputFormat.pdf => pdf(),
      OutputFormat.svg => svg(),
    };
    return File(path)..writeAsBytesSync(bytes);
  }

  /// Write `name-1.png`, `name-2.png`, … next to [path].
  List<File> savePages(String path, {double? density, double quality = 1.0}) {
    final dot = path.lastIndexOf('.');
    final stem = dot < 0 ? path : path.substring(0, dot);
    final extension = dot < 0 ? '.png' : path.substring(dot);
    final format = _formatFor(path, fallback: OutputFormat.png);
    final rendered = pages(format: format, density: density, quality: quality);
    return [
      for (var index = 0; index < rendered.length; index++)
        File('$stem-${index + 1}$extension')..writeAsBytesSync(rendered[index]),
    ];
  }

  // ── introspection ───────────────────────────────────────────────────────

  /// The computed layout tree.
  Map<String, Object?> layout() =>
      jsonDecode(_engine.dumpLayout(toJsonString())) as Map<String, Object?>;

  /// Dataset-style boxes at node, line or word granularity.
  Map<String, Object?> metadata([Granularity granularity = Granularity.node]) =>
      jsonDecode(_engine.dumpMetadata(toJsonString(), granularity.value))
          as Map<String, Object?>;

  static OutputFormat _formatFor(String path, {OutputFormat? fallback}) {
    final dot = path.lastIndexOf('.');
    final extension = dot < 0 ? '' : path.substring(dot).toLowerCase();
    return switch (extension) {
      '.png' => OutputFormat.png,
      '.jpg' || '.jpeg' => OutputFormat.jpeg,
      '.webp' => OutputFormat.webp,
      '.pdf' => OutputFormat.pdf,
      '.svg' => OutputFormat.svg,
      '.raw' || '.rgba' => OutputFormat.raw,
      _ => fallback ??
          (throw ArgumentError.value(
              path, 'path', 'cannot infer an output format')),
    };
  }
}

/// Wrap a node with render configuration.
///
///     render(root, density: 2).save('card.png');
Rendering render(
  Node root, {
  Engine? engine,
  num? width,
  num? height,
  String? background,
  num? density,
  num? pageHeight,
  Margin? margin,
  LastPageHeight? lastPageHeight,
  Node? header,
  Node? footer,
  List<FontSource>? fonts,
}) =>
    Rendering(root,
        engine: engine,
        width: width,
        height: height,
        background: background,
        density: density,
        pageHeight: pageHeight,
        margin: margin,
        lastPageHeight: lastPageHeight,
        header: header,
        footer: footer,
        fonts: fonts);
