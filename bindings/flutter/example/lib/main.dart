import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:sone_flutter/sone_flutter.dart' as s;

void main() => runApp(const SoneExampleApp());

class SoneExampleApp extends StatelessWidget {
  const SoneExampleApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
        title: 'sone',
        theme: ThemeData(colorSchemeSeed: Colors.teal, useMaterial3: true),
        home: const SoneDemo(),
      );
}

class SoneDemo extends StatefulWidget {
  const SoneDemo({super.key});

  @override
  State<SoneDemo> createState() => _SoneDemoState();
}

class _SoneDemoState extends State<SoneDemo> {
  late final Future<_Demo> _demo = _build();

  /// Reads the font out of the asset bundle, then renders three documents on a
  /// background isolate — the engine blocks whichever isolate calls it, so none
  /// of this touches the UI one.
  Future<_Demo> _build() async {
    final font = await s.loadFontAsset('Geist Mono', 'assets/GeistMono-Regular.ttf');

    final card = s.Column(
      gap: 20,
      padding: 20,
      width: 420,
      height: 300,
      bg: 'khaki',
      cornerRadius: 28,
      children: [
        s.Column(
          flex: 1,
          cornerRadius: 20,
          cornerSmoothing: 0.7,
          bg: 'white',
          justifyContent: s.JustifyContent.center,
          padding: 24,
          children: [
            s.Text('sone', font: 'Geist Mono', size: 34, color: '#14171a'),
            s.Text('running on Android',
                font: 'Geist Mono', size: 12, color: '#66707c'),
          ],
        ),
        s.Row(gap: 10, children: [
          s.Column(bg: 'lightgreen', size: 50, cornerRadius: 14),
          s.Column(bg: 'salmon', height: 50, cornerRadius: 14, flex: 1),
          s.Column(bg: 'orange', size: 50, cornerRadius: 14),
        ]),
      ],
    );

    final scripts = s.Column(
      gap: 14,
      padding: 24,
      width: 420,
      bg: '#ffffff',
      cornerRadius: 20,
      children: [
        s.Text('International text',
            font: 'Geist Mono', size: 20, color: '#14171a'),
        s.Text('The engine shapes complex scripts through HarfBuzz, and the '
            'same tree renders to PNG, PDF or SVG.',
            font: 'Geist Mono',
            size: 12,
            color: '#44505c',
            lineHeight: 1.7,
            align: s.TextAlign.justify),
      ],
    );

    return _Demo(
      card: await s.render(card, density: 2).pngAsync(fonts: [font]),
      scripts: await s.render(scripts, density: 2).pngAsync(fonts: [font]),
      // Proves the whole engine is here, not just the raster path.
      pdfBytes: (await s.render(card).pdfAsync(fonts: [font])).length,
      version: s.Engine.version,
    );
  }

  @override
  Widget build(BuildContext context) => Scaffold(
        appBar: AppBar(title: const Text('sone on Flutter')),
        body: FutureBuilder<_Demo>(
          future: _demo,
          builder: (context, snapshot) {
            if (snapshot.hasError) {
              return Padding(
                padding: const EdgeInsets.all(24),
                child: SelectableText('${snapshot.error}\n\n${snapshot.stackTrace}'),
              );
            }
            if (!snapshot.hasData) {
              return const Center(child: CircularProgressIndicator());
            }
            final demo = snapshot.data!;
            return ListView(
              padding: const EdgeInsets.all(16),
              children: [
                Text('engine ${demo.version} · PDF ${demo.pdfBytes} bytes',
                    style: Theme.of(context).textTheme.bodySmall),
                const SizedBox(height: 16),
                Image.memory(demo.card),
                const SizedBox(height: 16),
                Image.memory(demo.scripts),
              ],
            );
          },
        ),
      );
}

class _Demo {
  const _Demo({
    required this.card,
    required this.scripts,
    required this.pdfBytes,
    required this.version,
  });

  final Uint8List card;
  final Uint8List scripts;
  final int pdfBytes;
  final String version;
}
