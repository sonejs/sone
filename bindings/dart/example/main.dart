import 'package:sone/sone.dart' as s;

/// The named-argument form, end to end.
///
///     dart run example/main.dart
void main() {
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
        flex: 1,
        cornerRadius: 20,
        cornerSmoothing: 0.7,
        bg: 'white',
      ),
      s.Row(
        gap: 10,
        children: [
          s.Column(bg: 'lightgreen', size: 50, cornerRadius: 14, borderColor: 'teal', borderWidth: 2),
          s.Column(bg: 'salmon', height: 50, cornerRadius: 14, flex: 1),
          s.Column(bg: 'orange', size: 50, cornerRadius: 14),
        ],
      ),
    ],
  );

  final path = s.render(root, density: 2).save('card.png');
  print('wrote ${path.path}');
}
