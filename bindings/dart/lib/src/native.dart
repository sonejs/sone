import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

/// Result codes, mirroring the CLI's exit codes.
abstract final class SoneStatus {
  static const ok = 0;
  static const invalidArgument = 1;
  static const irError = 2;
  static const assetError = 3;
  static const renderError = 4;
}

/// The output formats the engine can encode.
enum OutputFormat {
  png(0),
  jpeg(1),
  webp(2),

  /// Raw RGBA pixels, row-major, unpremultiplied.
  raw(3),

  /// A PDF. With `pageHeight` set, one page per break and selectable text.
  pdf(4),
  svg(5);

  const OutputFormat(this.code);

  final int code;
}

/// `SoneRenderOptions` from `include/sone.h`.
final class SoneRenderOptions extends Struct {
  @Int32()
  external int format;

  @Float()
  external double density;

  @Float()
  external double quality;

  @Int32()
  external int strict;
}

/// An owned byte buffer. Release with `sone_buffer_free`.
final class SoneBuffer extends Struct {
  external Pointer<Uint8> data;

  @Size()
  external int len;

  @Size()
  external int capacity;
}

/// One buffer per page. Release the whole list with `sone_buffer_list_free`.
final class SoneBufferList extends Struct {
  external Pointer<SoneBuffer> items;

  @Size()
  external int len;

  @Size()
  external int capacity;
}

typedef _EngineNewC = Pointer<Void> Function(Pointer<Utf8>);
typedef _EngineNew = Pointer<Void> Function(Pointer<Utf8>);
typedef _EngineFreeC = Void Function(Pointer<Void>);
typedef _EngineFree = void Function(Pointer<Void>);
typedef _LastErrorC = Pointer<Utf8> Function(Pointer<Void>);
typedef _LastError = Pointer<Utf8> Function(Pointer<Void>);
typedef _RegisterBytesC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Uint8>, Size);
typedef _RegisterBytes = int Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Uint8>, int);
typedef _RegisterFileC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);
typedef _RegisterFile = int Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>);
typedef _HasFontC = Bool Function(Pointer<Void>, Pointer<Utf8>);
typedef _HasFont = bool Function(Pointer<Void>, Pointer<Utf8>);
typedef _FontFamiliesC = Int32 Function(Pointer<Void>, Pointer<SoneBuffer>);
typedef _FontFamilies = int Function(Pointer<Void>, Pointer<SoneBuffer>);
typedef _ResetFontsC = Void Function(Pointer<Void>);
typedef _ResetFonts = void Function(Pointer<Void>);
typedef _RenderC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, SoneRenderOptions, Pointer<SoneBuffer>);
typedef _Render = int Function(
    Pointer<Void>, Pointer<Utf8>, SoneRenderOptions, Pointer<SoneBuffer>);
typedef _RenderPagesC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, SoneRenderOptions, Pointer<SoneBufferList>);
typedef _RenderPages = int Function(
    Pointer<Void>, Pointer<Utf8>, SoneRenderOptions, Pointer<SoneBufferList>);
typedef _DumpLayoutC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<SoneBuffer>);
typedef _DumpLayout = int Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<SoneBuffer>);
typedef _DumpMetadataC = Int32 Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>, Pointer<SoneBuffer>);
typedef _DumpMetadata = int Function(
    Pointer<Void>, Pointer<Utf8>, Pointer<Utf8>, Pointer<SoneBuffer>);
typedef _BufferFreeC = Void Function(Pointer<SoneBuffer>);
typedef _BufferFree = void Function(Pointer<SoneBuffer>);
typedef _BufferListFreeC = Void Function(Pointer<SoneBufferList>);
typedef _BufferListFree = void Function(Pointer<SoneBufferList>);
typedef _VersionC = Pointer<Utf8> Function();
typedef _Version = Pointer<Utf8> Function();

/// The C ABI from `include/sone.h`. Nothing above this class sees a pointer.
class Native {
  Native(DynamicLibrary library)
      : engineNew =
            library.lookupFunction<_EngineNewC, _EngineNew>('sone_engine_new'),
        engineFree = library
            .lookupFunction<_EngineFreeC, _EngineFree>('sone_engine_free'),
        lastError = library
            .lookupFunction<_LastErrorC, _LastError>('sone_engine_last_error'),
        registerFont = library.lookupFunction<_RegisterBytesC, _RegisterBytes>(
            'sone_register_font'),
        registerFontFile =
            library.lookupFunction<_RegisterFileC, _RegisterFile>(
                'sone_register_font_file'),
        registerImage = library.lookupFunction<_RegisterBytesC, _RegisterBytes>(
            'sone_register_image'),
        hasFont = library.lookupFunction<_HasFontC, _HasFont>('sone_has_font'),
        fontFamilies = library.lookupFunction<_FontFamiliesC, _FontFamilies>(
            'sone_font_families'),
        resetFonts = library
            .lookupFunction<_ResetFontsC, _ResetFonts>('sone_reset_fonts'),
        renderJson =
            library.lookupFunction<_RenderC, _Render>('sone_render_json'),
        renderPages = library
            .lookupFunction<_RenderPagesC, _RenderPages>('sone_render_pages'),
        dumpLayout = library
            .lookupFunction<_DumpLayoutC, _DumpLayout>('sone_dump_layout'),
        dumpMetadata = library.lookupFunction<_DumpMetadataC, _DumpMetadata>(
            'sone_dump_metadata'),
        bufferFree = library
            .lookupFunction<_BufferFreeC, _BufferFree>('sone_buffer_free'),
        bufferListFree =
            library.lookupFunction<_BufferListFreeC, _BufferListFree>(
                'sone_buffer_list_free'),
        version = library.lookupFunction<_VersionC, _Version>('sone_version');

  final _EngineNew engineNew;
  final _EngineFree engineFree;
  final _LastError lastError;
  final _RegisterBytes registerFont;
  final _RegisterFile registerFontFile;
  final _RegisterBytes registerImage;
  final _HasFont hasFont;
  final _FontFamilies fontFamilies;
  final _ResetFonts resetFonts;
  final _Render renderJson;
  final _RenderPages renderPages;
  final _DumpLayout dumpLayout;
  final _DumpMetadata dumpMetadata;
  final _BufferFree bufferFree;
  final _BufferListFree bufferListFree;
  final _Version version;

  static Native? _instance;

  /// The loaded library, opened once per isolate group.
  static Native get instance => _instance ??= Native(_open());

  /// A full path to the library, or a directory holding it.
  static const pathVariable = 'SONE_NATIVE_LIBRARY';

  static String get _fileName {
    if (Platform.isWindows) return 'sone.dll';
    // iOS links the engine into the app binary rather than loading a dylib.
    if (Platform.isIOS) return '';
    if (Platform.isMacOS) return 'libsone.dylib';
    return 'libsone.so';
  }

  static DynamicLibrary _open() {
    // An empty name means the symbols are in the process already, which is how
    // iOS has to work.
    if (_fileName.isEmpty) {
      return DynamicLibrary.process();
    }
    for (final candidate in _candidates()) {
      try {
        return DynamicLibrary.open(candidate);
      } on ArgumentError {
        continue;
      }
    }
    throw StateError(
      'could not load the sone native library ($_fileName). Build it with '
      '`cargo build --release -p sone-ffi`, or set $pathVariable to its path.',
    );
  }

  /// An explicit hint first, then a `cargo build` in a checkout, then the
  /// loader's own search path — which is what a released package uses.
  static List<String> _candidates() {
    // On Android the library is inside the APK, unpacked into the app's
    // nativeLibraryDir, which is already on the loader's path. There is no
    // checkout to walk and no filesystem the app may read, so the bare name is
    // both the first and the only candidate.
    if (Platform.isAndroid || Platform.isIOS) {
      return [_fileName];
    }

    final found = <String>[];
    final hint = Platform.environment[pathVariable];
    if (hint != null && hint.isNotEmpty) {
      found.add(Directory(hint).existsSync() ? '$hint/$_fileName' : hint);
    }
    final root = checkoutRoot();
    if (root != null) {
      for (final profile in ['release', 'debug']) {
        found.add('$root/target/$profile/$_fileName');
      }
    }
    return [
      ...found.where((path) => File(path).existsSync()),
      _fileName,
    ];
  }

  /// The repository root, when this package is used from a checkout.
  ///
  /// Walks up from the working directory first and only then from the script:
  /// under `dart test` the script lives in a generated temp directory, so it is
  /// the less reliable of the two starting points.
  static String? checkoutRoot() {
    for (final start in _startingPoints()) {
      final found = _walkUp(start);
      if (found != null) return found;
    }
    return null;
  }

  static List<Directory> _startingPoints() {
    final starts = <Directory>[Directory.current];
    try {
      // Throws when the script URI is not a file — an embedder, or a snapshot.
      starts.add(Directory(Platform.script.toFilePath()).parent);
    } catch (_) {
      // Directory.current is enough.
    }
    return starts;
  }

  static String? _walkUp(Directory start) {
    if (!start.existsSync()) return null;
    // ignore: dead_code
    var directory = start;
    while (true) {
      if (File('${directory.path}/Cargo.toml').existsSync() &&
          Directory('${directory.path}/crates').existsSync()) {
        return directory.path;
      }
      final parent = directory.parent;
      if (parent.path == directory.path) return null;
      directory = parent;
    }
  }
}
