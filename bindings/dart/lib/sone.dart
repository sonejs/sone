/// sone — a declarative canvas layout engine with rich international text.
///
///     import 'package:sone/sone.dart' as s;
///
///     s.Font.load('Inter', 'fonts/Inter-Regular.ttf');
///
///     final root = s.Column(children: [
///       s.Text('Hello')
///         ..size(28)
///         ..weight('bold'),
///       s.Row(children: [
///         s.Column()..bg('salmon')..size(50)..rounded(14),
///         s.Column()..bg('orange')..size(50)..rounded(14),
///       ])..gap(10),
///     ])
///       ..gap(20)
///       ..padding(20)
///       ..bg('khaki')
///       ..cornerRadius(28);
///
///     s.render(root, density: 2).save('card.png');
///
/// The import prefix is the whole answer to the Flutter widget name collision:
/// `Column`, `Row` and `Text` all exist there too.
library;

export 'src/engine.dart'
    show
        Engine,
        Font,
        SoneException,
        IrException,
        AssetException,
        RenderException;
export 'src/keywords.dart';
export 'src/native.dart' show OutputFormat;
export 'src/node.dart'
    show Node, Dim, Track, LayoutProps, SpanStyleProps, TextBlockProps;
export 'src/nodes.dart';
export 'src/rendering.dart' show Rendering, Margin, FontSource, render;
