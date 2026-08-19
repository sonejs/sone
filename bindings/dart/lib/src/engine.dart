import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import 'native.dart';

/// The base for every sone failure.
class SoneException implements Exception {
  SoneException(this.message);

  final String message;

  @override
  String toString() => '$runtimeType: $message';
}

/// The IR document could not be parsed.
class IrException extends SoneException {
  IrException(super.message);
}

/// A font or an image could not be loaded.
class AssetException extends SoneException {
  AssetException(super.message);
}

/// Layout or rasterization failed.
class RenderException extends SoneException {
  RenderException(super.message);
}

/// Owns the font registry and the decoded-image cache.
///
/// Skia's font collection is shared inside an engine, so one engine renders one
/// document at a time. FFI calls block the isolate, so a Flutter app should
/// render inside `Isolate.run` — with its own engine, not this one.
class Engine {
  /// [baseDir] is the directory relative asset paths resolve against.
  Engine([String? baseDir]) : _native = Native.instance {
    final dir = (baseDir ?? Directory.current.path).toNativeUtf8();
    try {
      _handle = _native.engineNew(dir);
    } finally {
      calloc.free(dir);
    }
    if (_handle == nullptr) {
      throw SoneException('could not create a sone engine');
    }
  }

  final Native _native;
  late final Pointer<Void> _handle;
  bool _closed = false;

  static Engine? _default;

  /// The process-wide engine, used when no explicit one is passed.
  static Engine get instance => _default ??= Engine();

  /// The native library version.
  static String get version => Native.instance.version().toDartString();

  bool get isClosed => _closed;

  void close() {
    if (_closed) return;
    _closed = true;
    _native.engineFree(_handle);
  }

  // ── fonts and assets ──────────────────────────────────────────────────────

  /// Register a font family from raw TTF/OTF bytes.
  void registerFont(String name, List<int> data) =>
      _withBytes(name, data, _native.registerFont);

  /// Register a font family from a file.
  void registerFontFile(String name, String path) {
    final namePointer = name.toNativeUtf8();
    final pathPointer = path.toNativeUtf8();
    try {
      _check(_native.registerFontFile(_live, namePointer, pathPointer));
    } finally {
      calloc.free(namePointer);
      calloc.free(pathPointer);
    }
  }

  /// Make bytes available to documents as `asset:name`.
  void registerImage(String name, List<int> data) =>
      _withBytes(name, data, _native.registerImage);

  /// Whether a family has been registered.
  bool hasFont(String name) {
    final pointer = name.toNativeUtf8();
    try {
      return _native.hasFont(_live, pointer);
    } finally {
      calloc.free(pointer);
    }
  }

  /// Every registered family name.
  List<String> fontFamilies() {
    final json =
        utf8.decode(_buffer((out) => _native.fontFamilies(_live, out)));
    return (jsonDecode(json) as List<dynamic>).cast<String>();
  }

  /// Drop every registered font.
  void resetFonts() => _native.resetFonts(_live);

  // ── rendering ─────────────────────────────────────────────────────────────

  /// Render an IR document to bytes.
  Uint8List render(
    String document, {
    OutputFormat format = OutputFormat.png,
    double? density,
    double quality = 1.0,
    bool strict = false,
  }) =>
      _call(document, format, density, quality, strict, (json, options) {
        return _buffer(
            (out) => _native.renderJson(_live, json, options, out));
      });

  /// One raster image per page. Requires `pageHeight` in the document config.
  List<Uint8List> renderPages(
    String document, {
    OutputFormat format = OutputFormat.png,
    double? density,
    double quality = 1.0,
    bool strict = false,
  }) =>
      _call(document, format, density, quality, strict, (json, options) {
        final list = calloc<SoneBufferList>();
        try {
          _check(_native.renderPages(_live, json, options, list));
          return List<Uint8List>.generate(list.ref.len, (index) {
            final page = list.ref.items[index];
            return Uint8List.fromList(page.data.asTypedList(page.len));
          });
        } finally {
          _native.bufferListFree(list);
          calloc.free(list);
        }
      });

  /// The computed layout tree, as JSON.
  String dumpLayout(String document) => _withDocument(
      document,
      (json) =>
          utf8.decode(_buffer((out) => _native.dumpLayout(_live, json, out))));

  /// Dataset-style metadata, as JSON.
  String dumpMetadata(String document, String granularity) =>
      _withDocument(document, (json) {
        final pointer = granularity.toNativeUtf8();
        try {
          return utf8.decode(_buffer(
              (out) => _native.dumpMetadata(_live, json, pointer, out)));
        } finally {
          calloc.free(pointer);
        }
      });

  // ── internals ─────────────────────────────────────────────────────────────

  Pointer<Void> get _live {
    if (_closed) throw StateError('this engine has been closed');
    return _handle;
  }

  /// Owns the document string and the options struct for exactly one call.
  T _call<T>(
      String document,
      OutputFormat format,
      double? density,
      double quality,
      bool strict,
      T Function(Pointer<Utf8>, Pointer<SoneRenderOptions>) body) {
    final options = calloc<SoneRenderOptions>();
    options.ref
      ..format = format.code
      // Zero tells the engine to fall back to the document's own config.
      ..density = density ?? 0.0
      ..quality = quality
      ..strict = strict ? 1 : 0;
    try {
      return _withDocument(document, (json) => body(json, options));
    } finally {
      calloc.free(options);
    }
  }

  T _withDocument<T>(String document, T Function(Pointer<Utf8>) body) {
    final json = document.toNativeUtf8();
    try {
      return body(json);
    } finally {
      calloc.free(json);
    }
  }

  void _withBytes(String name, List<int> data,
      int Function(Pointer<Void>, Pointer<Utf8>, Pointer<Uint8>, int) call) {
    final namePointer = name.toNativeUtf8();
    final buffer = calloc<Uint8>(data.isEmpty ? 1 : data.length);
    try {
      buffer.asTypedList(data.length).setAll(0, data);
      _check(call(_live, namePointer, buffer, data.length));
    } finally {
      calloc.free(namePointer);
      calloc.free(buffer);
    }
  }

  Uint8List _buffer(int Function(Pointer<SoneBuffer>) call) {
    final out = calloc<SoneBuffer>();
    try {
      _check(call(out));
      final buffer = out.ref;
      if (buffer.data == nullptr || buffer.len == 0) return Uint8List(0);
      return Uint8List.fromList(buffer.data.asTypedList(buffer.len));
    } finally {
      _native.bufferFree(out);
      calloc.free(out);
    }
  }

  void _check(int status) {
    if (status == SoneStatus.ok) return;
    final pointer = _native.lastError(_handle);
    final message = pointer == nullptr
        ? 'sone failed with status $status'
        : pointer.toDartString();
    throw switch (status) {
      SoneStatus.invalidArgument => ArgumentError(message),
      SoneStatus.irError => IrException(message),
      SoneStatus.assetError => AssetException(message),
      _ => RenderException(message),
    };
  }
}

/// Font registration on the process-wide engine, for scripts that do not want
/// to own one. Skia carries no system fonts, so at least one family must be
/// registered before any text renders.
abstract final class Font {
  static void load(String name, String path) =>
      Engine.instance.registerFontFile(name, path);

  static void loadBytes(String name, List<int> data) =>
      Engine.instance.registerFont(name, data);

  static bool has(String name) => Engine.instance.hasFont(name);

  static List<String> families() => Engine.instance.fontFamilies();

  static void reset() => Engine.instance.resetFonts();
}
