use sone_core::compile::{compile, CompileCtx, NullAssets};
use sone_core::draw::{draw_tree, DrawCtx};
use sone_core::ir::Document;
use sone_core::layout::engine::layout;
use sone_core::testing::{FixedMetricsEngine, Op, RecordingPainter};

fn ops(json: &str) -> Vec<Op> {
    let doc = Document::from_json(json).unwrap();
    let assets = NullAssets;
    let mut ctx = CompileCtx::new(&assets);
    let root = compile(&doc.root, &mut ctx).unwrap().unwrap();
    let engine = FixedMetricsEngine::default();
    let (tree, state) = layout(&root, &engine, doc.config.width, doc.config.height);

    let mut painter = RecordingPainter::default();
    let draw_ctx = DrawCtx {
        state: &state,
        engine: &engine,
        debug_layout: false,
        debug_text: false,
    };
    draw_tree(&mut painter, &root, &tree, &draw_ctx);
    painter.ops()
}

fn index_of(ops: &[Op], pred: impl Fn(&Op) -> bool) -> Option<usize> {
    ops.iter().position(pred)
}

#[test]
fn shadow_then_clip_then_background_then_border() {
    let ops = ops(r##"{"sone":1,"root":{"type":"column","props":{
             "width":100,"height":50,"background":["red"],
             "shadows":["2px 2px 4px black"],"borderWidth":2,"borderColor":"blue"}}}"##);

    let shadow =
        index_of(&ops, |o| matches!(o, Op::DrawPath(p) if p.shadow.is_some())).expect("shadow");
    let clip = index_of(&ops, |o| matches!(o, Op::ClipPath)).expect("clip");
    let background = index_of(&ops, |o| matches!(o, Op::DrawRect(..))).expect("background");
    let border = ops
        .iter()
        .rposition(|o| matches!(o, Op::DrawPath(p) if p.stroke.is_some()))
        .expect("border");

    assert!(shadow < clip, "the shadow is painted before the clip");
    assert!(clip < background, "the background is clipped");
    assert!(
        background < border,
        "the border is painted over the background"
    );
}

#[test]
fn no_background_means_no_clip_and_no_shadow() {
    let ops = ops(
        r#"{"sone":1,"root":{"type":"column","props":{"width":10,"height":10,"shadows":["2px 2px 4px black"]}}}"#,
    );
    assert!(!ops
        .iter()
        .any(|o| matches!(o, Op::DrawRect(..) | Op::ClipPath)));
}

#[test]
fn children_paint_after_their_parent_background() {
    let ops = ops(
        r##"{"sone":1,"root":{"type":"column","props":{"background":["red"],"width":100,"height":50},
             "children":[{"type":"column","props":{"background":["blue"],"width":10,"height":10}}]}}"##,
    );
    let rects: Vec<&Op> = ops
        .iter()
        .filter(|o| matches!(o, Op::DrawRect(..)))
        .collect();
    assert_eq!(rects.len(), 2);
    match (rects[0], rects[1]) {
        (Op::DrawRect(a, _), Op::DrawRect(b, _)) => {
            assert_eq!(a.width(), 100.0);
            assert_eq!(b.width(), 10.0);
        }
        _ => unreachable!(),
    }
}

#[test]
fn group_opacity_opens_exactly_one_layer() {
    let ops = ops(
        r##"{"sone":1,"root":{"type":"column","props":{"opacity":0.5,"background":["red"],"width":10,"height":10}}}"##,
    );
    let layers: Vec<&Op> = ops
        .iter()
        .filter(|o| matches!(o, Op::SaveLayer(_)))
        .collect();
    assert_eq!(layers.len(), 1);
    match layers[0] {
        Op::SaveLayer(spec) => assert_eq!(spec.alpha, Some(0.5)),
        _ => unreachable!(),
    }
}

#[test]
fn a_fully_opaque_node_opens_no_layer() {
    let ops = ops(
        r##"{"sone":1,"root":{"type":"column","props":{"background":["red"],"width":10,"height":10}}}"##,
    );
    assert!(!ops.iter().any(|o| matches!(o, Op::SaveLayer(_))));
}

#[test]
fn transforms_are_applied_about_the_center() {
    let ops = ops(
        r#"{"sone":1,"root":{"type":"column","props":{"width":100,"height":50,"rotation":30,"translateX":5}}}"#,
    );
    assert!(ops.contains(&Op::Translate(5.0, 0.0)));
    assert!(ops.contains(&Op::Rotate(30.0, 50.0, 25.0)));
}

#[test]
fn scale_brackets_the_center() {
    let ops = ops(
        r#"{"sone":1,"root":{"type":"column","props":{"width":100,"height":50,"scale":[2,3]}}}"#,
    );
    let i = index_of(&ops, |o| matches!(o, Op::Scale(..))).expect("scale");
    assert_eq!(ops[i - 1], Op::Translate(50.0, 25.0));
    assert_eq!(ops[i], Op::Scale(2.0, 3.0));
    assert_eq!(ops[i + 1], Op::Translate(-50.0, -25.0));
}

#[test]
fn uniform_borders_use_the_clipped_double_stroke() {
    let ops = ops(
        r#"{"sone":1,"root":{"type":"column","props":{"width":100,"height":50,"borderWidth":3,"borderColor":"black"}}}"#,
    );
    let stroke = ops
        .iter()
        .find_map(|o| match o {
            Op::DrawPath(p) => p.stroke.clone(),
            _ => None,
        })
        .expect("border stroke");
    assert_eq!(stroke.width, 6.0);
    assert!(ops.iter().any(|o| matches!(o, Op::ClipPath)));
}

#[test]
fn per_side_borders_are_drawn_as_lines() {
    let ops = ops(
        r#"{"sone":1,"root":{"type":"column","props":{"width":100,"height":50,
             "borderTopWidth":2,"borderBottomWidth":4,"borderColor":"black"}}}"#,
    );
    let lines: Vec<&Op> = ops
        .iter()
        .filter(|o| matches!(o, Op::DrawLine(..)))
        .collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn text_runs_are_drawn_in_reading_order() {
    let ops = ops(r#"{"sone":1,"root":{"type":"text","props":{"size":10},
             "inline":["one ",{"type":"span","props":{"color":"red"},"inline":["two"]}]}}"#);
    let texts: Vec<String> = ops
        .iter()
        .filter_map(|o| match o {
            Op::DrawText { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, vec!["one ", "two"]);
}

#[test]
fn decorations_bracket_the_glyphs() {
    let ops = ops(
        r#"{"sone":1,"root":{"type":"text","props":{"size":10,"underline":1,"lineThrough":1},"inline":["x"]}}"#,
    );
    let underline = index_of(&ops, |o| matches!(o, Op::DrawRect(..))).expect("underline");
    let glyphs = index_of(&ops, |o| matches!(o, Op::DrawText { .. })).expect("glyphs");
    let strike = ops
        .iter()
        .rposition(|o| matches!(o, Op::DrawRect(..)))
        .expect("strike");
    assert!(
        underline < glyphs,
        "the underline is painted under the glyphs"
    );
    assert!(
        glyphs < strike,
        "the line-through is painted over the glyphs"
    );
}

#[test]
fn table_grid_lines_are_one_path() {
    let ops = ops(
        r#"{"sone":1,"root":{"type":"table","props":{"borderColor":"black"},"children":[
             {"type":"table-row","children":[
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["a"]}]},
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["b"]}]}]},
             {"type":"table-row","children":[
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["c"]}]},
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["d"]}]}]}]}}"#,
    );
    let strokes = ops
        .iter()
        .filter(|o| matches!(o, Op::DrawPath(p) if p.stroke.is_some()))
        .count();
    assert_eq!(strokes, 1, "the whole grid is stroked in one call");
    // The grid is painted after the cell content.
    let last_text = ops
        .iter()
        .rposition(|o| matches!(o, Op::DrawText { .. }))
        .unwrap();
    let grid = ops
        .iter()
        .rposition(|o| matches!(o, Op::DrawPath(p) if p.stroke.is_some()))
        .unwrap();
    assert!(last_text < grid);
}

#[test]
fn clip_group_clips_before_its_children() {
    let ops = ops(
        r##"{"sone":1,"root":{"type":"clip-group","props":{"clipPath":"M0,0 L10,0 L10,10 Z","width":10,"height":10},
             "children":[{"type":"column","props":{"background":["red"],"width":10,"height":10}}]}}"##,
    );
    let clip = index_of(&ops, |o| matches!(o, Op::ClipPath)).expect("clip");
    let rect = index_of(&ops, |o| matches!(o, Op::DrawRect(..))).expect("child background");
    assert!(clip < rect);
}

#[test]
fn save_and_restore_stay_balanced() {
    let ops = ops(
        r##"{"sone":1,"root":{"type":"column","props":{"background":["red"],"opacity":0.5,"width":50,"height":50},
             "children":[{"type":"column","props":{"background":["blue"],"width":10,"height":10}}]}}"##,
    );
    let saves = ops.iter().filter(|o| matches!(o, Op::Save)).count();
    let restores = ops
        .iter()
        .filter(|o| matches!(o, Op::RestoreToCount(_)))
        .count();
    assert_eq!(saves, restores);
}
