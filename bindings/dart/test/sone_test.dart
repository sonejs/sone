import 'dart:convert';
import 'dart:io';

import 'package:sone/sone.dart' as s;
import 'package:test/test.dart';

/// The checkout root, found by walking up from this test file.
String findRoot() {
  var directory = Directory.current;
  while (true) {
    if (File('${directory.path}/Cargo.toml').existsSync() &&
        Directory('${directory.path}/crates').existsSync()) {
      return directory.path;
    }
    final parent = directory.parent;
    if (parent.path == directory.path) {
      throw StateError('could not find the repository root');
    }
    directory = parent;
  }
}

final root = findRoot();
final fontPath = '$root/fixtures/font/GeistMono-Regular.ttf';
const family = 'Geist Mono';

Map<String, Object?> propsOf(s.Node node) =>
    (node.toIr()['props'] ?? <String, Object?>{}) as Map<String, Object?>;

void main() {
  // ── the builder, which touches no native code ───────────────────────────

  group('builder', () {
    test('everything is a named argument', () {
      final root = s.Column(
        gap: 20,
        padding: 20,
        width: 420,
        height: 300,
        bg: 'khaki',
        cornerRadius: 28,
        borderColor: 'chocolate',
        borderWidth: 4,
        rotate: 20,
        children: [
          s.Column(
              flex: 1, cornerRadius: 20, cornerSmoothing: 0.7, bg: 'white'),
          s.Row(gap: 10, children: [
            s.Column(bg: 'lightgreen', size: 50, cornerRadius: 14),
            s.Column(bg: 'salmon', height: 50, cornerRadius: 14, flex: 1),
          ]),
        ],
      );

      final ir = root.toIr();
      expect(ir['props'], containsPair('gap', 20));
      expect(ir['props'], containsPair('width', 420));
      expect(ir['props'], containsPair('background', ['khaki']));
      expect(ir['props'], containsPair('cornerRadius', [28]));
      expect(ir['props'], containsPair('rotation', 20));
      expect((ir['children']! as List).length, 2);
    });

    test('cascades still work, and mix with named arguments', () {
      // The setters never went away; named arguments call straight into them.
      final node = s.Column(gap: 20)
        ..padding(10, 20)
        ..bg('khaki');
      expect(propsOf(node)['gap'], 20);
      expect(propsOf(node)['paddingRight'], 20);
    });

    test('filters keep the order a cascade gives them', () {
      // Named arguments apply filters in a fixed order; cascades are how you
      // choose one.
      final node = s.Column()
        ..grayscale(0.5)
        ..blur(4);
      expect(propsOf(node)['filters'], ['grayscale(0.5)', 'blur(4px)']);
    });

    test('collection-for and collection-if generate children', () {
      final rows = ['a', 'b', 'c'];
      final table = s.Table(children: [
        for (final row in rows)
          s.TableRow(children: [
            s.TableCell(children: [s.Text(row)])
          ]),
        if (rows.isEmpty) s.TableRow(children: [s.TableCell()]),
      ]);
      expect((table.toIr()['children']! as List).length, 3);
    });

    test('dims take numbers, percentages and auto', () {
      final props =
          propsOf(s.Column(width: 100, minWidth: '50%', maxWidth: 'auto'));
      expect(props['width'], 100);
      expect(props['minWidth'], '50%');
      expect(props['maxWidth'], 'auto');
    });

    test('a bad dim is rejected at the call site', () {
      expect(() => s.Column(width: 'wide'), throwsArgumentError);
    });

    test('size with one argument is a square', () {
      final props = propsOf(s.Column(size: 50));
      expect(props['width'], 50);
      expect(props['height'], 50);
    });

    test('box shorthand follows CSS', () {
      final props = propsOf(s.Column()..padding(10, 20));
      expect(props['paddingTop'], 10);
      expect(props['paddingRight'], 20);
      expect(props['paddingBottom'], 10);
      expect(props['paddingLeft'], 20);
      expect(props.containsKey('padding'), isFalse);
    });

    test('one value uses the shorthand property', () {
      expect(propsOf(s.Column(margin: 12))['margin'], 12);
    });

    test('keywords are enums', () {
      final props = propsOf(s.Row(
        justifyContent: s.JustifyContent.spaceBetween,
        alignItems: s.AlignItems.center,
      ));
      expect(props['justifyContent'], 'space-between');
      expect(props['alignItems'], 'center');
    });

    test('background layers accumulate and take a photo', () {
      final props =
          propsOf(s.Column(backgrounds: ['red', s.Photo('wall.png')]));
      final layers = props['background']! as List;
      expect(layers[0], 'red');
      expect((layers[1]! as Map)['type'], 'photo');
    });

    test('Text.size is the font size, not the box size', () {
      final props = propsOf(s.Text('Hello', size: 28));
      expect(props['size'], 28);
      expect(props.containsKey('width'), isFalse);
    });

    test('Text.size refuses the box-sizing call shape', () {
      expect(() => s.Text('x')..size(20, 30), throwsArgumentError);
    });

    test('text takes content and spans', () {
      final node = s.Text([
        'Hello ',
        s.Span('world', weight: 'bold', color: 'salmon'),
      ]);
      final inline = node.toIr()['inline']! as List;
      expect(inline[0], 'Hello ');
      expect((inline[1]! as Map)['type'], 'span');
    });

    test('a decoration colour can be explicitly null', () {
      // A null named argument means "unset", so the explicit null that means
      // "use the text colour" is a cascade.
      final props = propsOf(s.Text('x')
        ..underline()
        ..underlineColor());
      expect(props.containsKey('underlineColor'), isTrue);
      expect(props['underlineColor'], isNull);
    });

    test('grid tracks accept fr and auto', () {
      final props = propsOf(s.Grid(columns: ['1fr', 'auto', 120]));
      expect(props['columns'], ['1fr', 'auto', 120]);
    });

    test('photo bytes become a data url', () {
      final props = propsOf(s.Photo([1, 2, 3]));
      expect(props['src'], startsWith('data:application/octet-stream;base64,'));
    });

    test('the document carries the schema version', () {
      final document = s.render(s.Column()).toIr();
      expect(document['sone'], 1);
      expect(document.containsKey('config'), isFalse);
    });

    test('pagination tokens are passed through untouched', () {
      final json = s
          .render(s.Column(),
              pageHeight: 800, header: s.Text('Page {pageNumber}'))
          .toJsonString();
      expect(json, contains('{pageNumber}'));
    });

    test('non-ascii text survives unescaped', () {
      expect(s.render(s.Text('អក្សរ')).toJsonString(), contains('អក្សរ'));
    });
  });

  // ── everything that crosses the C ABI ───────────────────────────────────

  group('engine', () {
    late s.Engine engine;

    setUp(() {
      engine = s.Engine(root)..registerFontFile(family, fontPath);
    });

    tearDown(() => engine.close());

    test('renders a png', () {
      final png = s.render(s.Column(size: 16, bg: 'red'), engine: engine).png();
      expect(png.sublist(0, 4), [0x89, 0x50, 0x4E, 0x47]);
    });

    test('density scales the raster', () {
      s.Column node() => s.Column(size: 10, bg: 'red');
      // Raw is 4 bytes per pixel, so the byte count is the pixel count.
      expect(s.render(node(), engine: engine).raw().length, 10 * 10 * 4);
      expect(
          s.render(node(), engine: engine).raw(density: 2).length, 20 * 20 * 4);
    });

    test('renders every format', () {
      final rendering =
          s.render(s.Column(size: 16, bg: 'teal'), engine: engine);
      expect(rendering.jpeg(quality: 0.8), isNotEmpty);
      expect(rendering.webp(), isNotEmpty);
      expect(utf8.decode(rendering.pdf().sublist(0, 4)), '%PDF');
      expect(utf8.decode(rendering.svg()), contains('<svg'));
    });

    test('one page per declared break', () {
      final root = s.Column(children: [
        s.Column(height: 60, bg: 'red'),
        s.Column(height: 60, bg: 'green', pageBreak: s.PageBreakMode.before),
        s.Column(height: 60, bg: 'blue', pageBreak: s.PageBreakMode.before),
      ]);
      final pages =
          s.render(root, engine: engine, width: 40, pageHeight: 200).pages();
      expect(pages.length, 3);
    });

    test('the font registry round trips', () {
      final fresh = s.Engine(root);
      addTearDown(fresh.close);
      expect(fresh.hasFont(family), isFalse);
      fresh.registerFontFile(family, fontPath);
      expect(fresh.hasFont(family), isTrue);
      expect(fresh.fontFamilies(), contains(family));
      fresh.resetFonts();
      expect(fresh.hasFont(family), isFalse);
      fresh.registerFont(family, File(fontPath).readAsBytesSync());
      expect(fresh.hasFont(family), isTrue);
    });

    test('registered images resolve as assets', () {
      final png = s
          .render(
              s.Column()
                ..size(8)
                ..bg('red'),
              engine: engine)
          .png();
      engine.registerImage('logo', png);
      expect(s.render(s.Photo('asset:logo')..size(8), engine: engine).png(),
          isNotEmpty);
    });

    test('layout comes back as a tree', () {
      final node = s.Column(padding: 5, children: [
        s.Column(size: 20, tag: 'inner'),
      ]);
      final layout = s.render(node, engine: engine).layout();
      expect(layout['width'], 30);
      expect((layout['children']! as List).first, containsPair('tag', 'inner'));
    });

    test('metadata honours granularity', () {
      final rendering = s.render(s.Text('hello world', font: family, size: 12),
          engine: engine);
      expect(rendering.metadata(), isA<Map<String, Object?>>());
      expect(
          rendering.metadata(s.Granularity.word), isA<Map<String, Object?>>());
    });

    test('a bad document is an IR error', () {
      expect(
          () => engine.render('{"sone":99,"root":{"type":"column"}}'),
          throwsA(isA<s.IrException>().having((e) => e.message, 'message',
              contains('unsupported IR version'))));
    });

    test('a missing font file is an asset error', () {
      expect(() => engine.registerFontFile('Nope', 'does/not/exist.ttf'),
          throwsA(isA<s.AssetException>()));
    });

    test('using a closed engine throws rather than crashing', () {
      final closed = s.Engine(root)..close();
      closed.close();
      expect(() => closed.hasFont(family), throwsStateError);
    });

    test('save infers the format from the extension', () {
      final directory = Directory.systemTemp.createTempSync('sone-dart');
      addTearDown(() => directory.deleteSync(recursive: true));
      final file = s
          .render(s.Column(size: 16, bg: 'red'), engine: engine)
          .save('${directory.path}/card.pdf');
      expect(utf8.decode(file.readAsBytesSync().sublist(0, 4)), '%PDF');
    });

    // The gate every binding owes: the same document must come out of this
    // binding byte for byte the way it comes out of `sone-cli`.
    test('matches the CLI byte for byte', () {
      final node = s.Column(
        gap: 20,
        padding: 20,
        width: 420,
        height: 200,
        bg: 'khaki',
        cornerRadius: 28,
        children: [
          s.Text(
            ['Hello ', s.Span('world', weight: 'bold', color: '#c0392b')],
            font: family,
            size: 24,
            lineHeight: 1.4,
          ),
          s.Row(gap: 10, children: [
            s.Column(bg: 'lightgreen', size: 50, cornerRadius: 14),
            s.Column(bg: 'salmon', height: 50, cornerRadius: 14, flex: 1),
          ]),
        ],
      );

      // An absolute src, because the CLI resolves a document's assets against
      // the document's own directory and the engine resolves them against its
      // base directory — the two only agree when the path is absolute.
      final rendering = s.render(node,
          engine: engine, density: 2, fonts: [s.FontSource(family, fontPath)]);

      final directory = Directory.systemTemp.createTempSync('sone-parity');
      addTearDown(() => directory.deleteSync(recursive: true));
      final document = '${directory.path}/doc.json';
      File(document).writeAsStringSync(rendering.toJsonString(pretty: true));
      final fromCli = '${directory.path}/cli.png';

      final result = Process.runSync(
        'cargo',
        [
          'run',
          '-q',
          '-p',
          'sone-cli',
          '--',
          'render',
          document,
          '--density',
          '2',
          '-o',
          fromCli
        ],
        workingDirectory: root,
      );
      expect(result.exitCode, 0, reason: result.stderr.toString());
      expect(rendering.png(), File(fromCli).readAsBytesSync());
    });
  });
}
