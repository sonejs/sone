/// Flutter packaging for sone.
///
/// The API is `package:sone` — this library re-exports it, adds the native
/// library for Android, and supplies the two things a Flutter app needs that a
/// command-line program does not: fonts read from the asset bundle, and
/// rendering that does not block the UI isolate.
///
/// ```dart
/// import 'package:sone_flutter/sone_flutter.dart' as s;
///
/// final font = await s.loadFontAsset('Inter', 'assets/Inter-Regular.ttf');
///
/// final root = s.Column(
///   gap: 20, padding: 20, bg: 'khaki', cornerRadius: 28,
///   children: [s.Text('Hello', font: 'Inter', size: 28)],
/// );
///
/// final png = await s.render(root, density: 2).pngAsync(fonts: [font]);
/// // ... Image.memory(png)
/// ```
library;

import 'dart:isolate';
import 'dart:typed_data';

import 'package:flutter/services.dart' show rootBundle;
import 'package:sone/sone.dart';

export 'package:sone/sone.dart';

/// A font as bytes, which is what can cross an isolate boundary.
///
/// An [Engine] cannot: it owns a native pointer, so a background isolate has to
/// build its own and be handed the font data.
class SoneFont {
  const SoneFont(this.family, this.bytes);

  final String family;
  final Uint8List bytes;
}

/// Reads a font out of the Flutter asset bundle.
///
/// Skia carries no system fonts — not on Android either — so at least one
/// family has to be registered before any text renders.
Future<SoneFont> loadFontAsset(String family, String assetKey) async {
  final data = await rootBundle.load(assetKey);
  return SoneFont(family, data.buffer.asUint8List());
}

/// Registers a font on an engine in this isolate.
void registerFont(SoneFont font, {Engine? engine}) =>
    (engine ?? Engine.instance).registerFont(font.family, font.bytes);

/// Renders on a background isolate, with an engine of its own.
///
/// Every FFI call blocks the isolate that makes it, and a render is long enough
/// to drop frames. Only sendable values cross: the document as JSON, and fonts
/// as bytes.
Future<Uint8List> renderIsolated(
  String document, {
  List<SoneFont> fonts = const [],
  OutputFormat format = OutputFormat.png,
  double? density,
  double quality = 1.0,
}) {
  final payload = <List<Object>>[
    for (final font in fonts) [font.family, font.bytes],
  ];
  return Isolate.run(() {
    final engine = Engine();
    try {
      for (final font in payload) {
        engine.registerFont(font[0] as String, font[1] as Uint8List);
      }
      return engine.render(document,
          format: format, density: density, quality: quality);
    } finally {
      engine.close();
    }
  });
}

/// One raster image per page, on a background isolate.
Future<List<Uint8List>> renderPagesIsolated(
  String document, {
  List<SoneFont> fonts = const [],
  OutputFormat format = OutputFormat.png,
  double? density,
}) {
  final payload = <List<Object>>[
    for (final font in fonts) [font.family, font.bytes],
  ];
  return Isolate.run(() {
    final engine = Engine();
    try {
      for (final font in payload) {
        engine.registerFont(font[0] as String, font[1] as Uint8List);
      }
      return engine.renderPages(document, format: format, density: density);
    } finally {
      engine.close();
    }
  });
}

/// The async outputs, on any [Rendering].
extension SoneFlutterRendering on Rendering {
  /// A PNG, rendered off the UI isolate.
  Future<Uint8List> pngAsync({
    List<SoneFont> fonts = const [],
    double? density,
  }) =>
      renderIsolated(toJsonString(),
          fonts: fonts, format: OutputFormat.png, density: density);

  /// A JPEG, rendered off the UI isolate.
  Future<Uint8List> jpegAsync({
    List<SoneFont> fonts = const [],
    double? density,
    double quality = 1.0,
  }) =>
      renderIsolated(toJsonString(),
          fonts: fonts,
          format: OutputFormat.jpeg,
          density: density,
          quality: quality);

  /// A PDF, rendered off the UI isolate.
  Future<Uint8List> pdfAsync({List<SoneFont> fonts = const []}) =>
      renderIsolated(toJsonString(), fonts: fonts, format: OutputFormat.pdf);

  /// One raster image per page, rendered off the UI isolate.
  Future<List<Uint8List>> pagesAsync({
    List<SoneFont> fonts = const [],
    double? density,
  }) =>
      renderPagesIsolated(toJsonString(), fonts: fonts, density: density);
}
