use sone_core::compile::{compile, CompileCtx, NullAssets};
use sone_core::ir::Document;
use sone_core::layout::engine::{layout, BoxLayout};
use sone_core::testing::FixedMetricsEngine;

fn lay(json: &str, w: Option<f32>, h: Option<f32>) -> BoxLayout {
    let doc = Document::from_json(json).unwrap();
    let assets = NullAssets;
    let mut ctx = CompileCtx::new(&assets);
    let root = compile(&doc.root, &mut ctx).unwrap().unwrap();
    let engine = FixedMetricsEngine::default();
    layout(&root, &engine, w, h).0
}

#[test]
fn column_stacks_children_vertically() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"column","props":{"gap":10},"children":[
             {"type":"column","props":{"width":50,"height":20}},
             {"type":"column","props":{"width":80,"height":30}}]}}"#,
        None,
        None,
    );
    assert_eq!((b.width, b.height), (80.0, 60.0));
    assert_eq!((b.children[0].x, b.children[0].y), (0.0, 0.0));
    assert_eq!((b.children[1].x, b.children[1].y), (0.0, 30.0));
}

#[test]
fn row_stacks_children_horizontally() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"row","props":{"gap":4},"children":[
             {"type":"column","props":{"width":50,"height":20}},
             {"type":"column","props":{"width":30,"height":40}}]}}"#,
        None,
        None,
    );
    assert_eq!((b.width, b.height), (84.0, 40.0));
    assert_eq!(b.children[1].x, 54.0);
}

#[test]
fn padding_and_border_inset_children() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"column","props":{"padding":10,"borderWidth":2},
             "children":[{"type":"column","props":{"width":40,"height":10}}]}}"#,
        None,
        None,
    );
    assert_eq!(b.width, 64.0);
    assert_eq!(b.padding.left, 10.0);
    assert_eq!(b.border.left, 2.0);
    assert_eq!((b.children[0].x, b.children[0].y), (12.0, 12.0));
}

#[test]
fn percentage_width_resolves_against_the_parent() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"column","props":{"width":200},
             "children":[{"type":"column","props":{"width":"50%","height":10}}]}}"#,
        None,
        None,
    );
    assert_eq!(b.children[0].width, 100.0);
}

#[test]
fn flex_grow_shares_the_remaining_space() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"row","props":{"width":100,"height":10},"children":[
             {"type":"column","props":{"flexGrow":1}},
             {"type":"column","props":{"flexGrow":3}}]}}"#,
        None,
        None,
    );
    assert_eq!(b.children[0].width, 25.0);
    assert_eq!(b.children[1].width, 75.0);
}

#[test]
fn yoga_default_shrink_keeps_children_at_their_size() {
    // With taffy's own default (shrink 1) the children would compress to 50 each.
    let b = lay(
        r#"{"sone":1,"root":{"type":"row","props":{"width":100,"height":10},"children":[
             {"type":"column","props":{"width":80}},
             {"type":"column","props":{"width":80}}]}}"#,
        None,
        None,
    );
    assert_eq!(b.children[0].width, 80.0);
    assert_eq!(b.children[1].width, 80.0);
}

#[test]
fn absolute_children_are_positioned_by_inset() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"column","props":{"width":100,"height":100},"children":[
             {"type":"column","props":{"position":"absolute","left":10,"top":20,"width":5,"height":5}}]}}"#,
        None,
        None,
    );
    assert_eq!((b.children[0].x, b.children[0].y), (10.0, 20.0));
}

#[test]
fn text_measures_from_the_engine() {
    // FixedMetricsEngine: advance = size per char, ascent .8, descent .2
    let b = lay(
        r#"{"sone":1,"root":{"type":"text","props":{"size":10},"inline":["abcd"]}}"#,
        None,
        None,
    );
    assert_eq!(b.width, 40.0);
    assert_eq!(b.height, 10.0);
}

#[test]
fn text_wraps_to_the_available_width() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"column","props":{"width":60},
             "children":[{"type":"text","props":{"size":10},"inline":["hello world"]}]}}"#,
        None,
        None,
    );
    assert_eq!(b.children[0].height, 20.0);
}

#[test]
fn grid_splits_fr_tracks_evenly() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"grid","props":{"width":300,"columns":["1fr","2fr"],"columnGap":0},
             "children":[
               {"type":"column","props":{"height":10}},
               {"type":"column","props":{"height":10}}]}}"#,
        None,
        None,
    );
    assert_eq!(b.children.len(), 2);
    assert_eq!(b.children[0].width, 100.0);
    assert_eq!(b.children[1].width, 200.0);
    assert_eq!(b.children[1].x, 100.0);
}

#[test]
fn grid_auto_places_row_major() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"grid","props":{"columns":[50,50]},"children":[
             {"type":"column","props":{"height":10}},
             {"type":"column","props":{"height":10}},
             {"type":"column","props":{"height":10}}]}}"#,
        None,
        None,
    );
    assert_eq!((b.children[0].x, b.children[0].y), (0.0, 0.0));
    assert_eq!((b.children[1].x, b.children[1].y), (50.0, 0.0));
    assert_eq!((b.children[2].x, b.children[2].y), (0.0, 10.0));
}

#[test]
fn grid_gaps_offset_tracks() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"grid","props":{"columns":[50,50],"columnGap":8,"rowGap":6},
             "children":[
               {"type":"column","props":{"height":10}},
               {"type":"column","props":{"height":10}},
               {"type":"column","props":{"height":10}}]}}"#,
        None,
        None,
    );
    assert_eq!(b.children[1].x, 58.0);
    assert_eq!(b.children[2].y, 16.0);
}

#[test]
fn table_columns_equalize_across_rows() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"table","children":[
             {"type":"table-row","children":[
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["aaaaaa"]}]},
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["b"]}]}]},
             {"type":"table-row","children":[
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["c"]}]},
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["dddd"]}]}]}]}}"#,
        None,
        None,
    );
    let r0 = &b.children[0];
    let r1 = &b.children[1];
    assert_eq!(r0.children[0].width, r1.children[0].width);
    assert_eq!(r0.children[1].width, r1.children[1].width);
    assert_eq!(r0.children[0].width, 60.0);
    assert_eq!(r0.children[1].width, 40.0);
}

#[test]
fn table_rows_share_a_height() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"table","children":[
             {"type":"table-row","children":[
               {"type":"table-cell","children":[{"type":"text","props":{"size":20},"inline":["a"]}]},
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["b"]}]}]}]}}"#,
        None,
        None,
    );
    let row = &b.children[0];
    assert_eq!(row.children[0].height, row.children[1].height);
}

#[test]
fn table_colspan_widens_the_spanned_cell() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"table","children":[
             {"type":"table-row","children":[
               {"type":"table-cell","props":{"colspan":2},"children":[{"type":"text","props":{"size":10},"inline":["aaaaaaaa"]}]}]},
             {"type":"table-row","children":[
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["b"]}]},
               {"type":"table-cell","children":[{"type":"text","props":{"size":10},"inline":["c"]}]}]}]}}"#,
        None,
        None,
    );
    let spanned = b.children[0].children[0].width;
    let a = b.children[1].children[0].width;
    let c = b.children[1].children[1].width;
    assert!((spanned - (a + c)).abs() < 0.51, "{spanned} vs {a}+{c}");
}

#[test]
fn list_items_become_marker_plus_content() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"list","props":{"listStyle":"decimal","markerGap":6},"children":[
             {"type":"list-item","children":[{"type":"text","props":{"size":10},"inline":["one"]}]},
             {"type":"list-item","children":[{"type":"text","props":{"size":10},"inline":["two"]}]}]}}"#,
        None,
        None,
    );
    assert_eq!(b.children.len(), 2);
    let item = &b.children[0];
    assert_eq!(item.children.len(), 2);
    // The marker is its own Text node at the default 11px size: "1." = 22px,
    // then the 6px marker gap.
    assert_eq!(item.children[0].width, 22.0);
    assert_eq!(item.children[1].x, 28.0);
}

#[test]
fn display_none_removes_a_child_from_flow() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"column","children":[
             {"type":"column","props":{"display":"none","width":50,"height":50}},
             {"type":"column","props":{"width":10,"height":10}}]}}"#,
        None,
        None,
    );
    assert_eq!(b.height, 10.0);
}

#[test]
fn root_width_constrains_the_tree() {
    let b = lay(
        r#"{"sone":1,"root":{"type":"column","children":[{"type":"column","props":{"height":10}}]}}"#,
        Some(320.0),
        None,
    );
    assert_eq!(b.width, 320.0);
}

#[test]
fn content_box_text_wraps_at_the_content_width() {
    // width 20 with padding 5 → outer 30, content 20 → 2 chars per line at
    // size 10, so 5 characters take 3 lines.
    let b = lay(
        r#"{"sone":1,"root":{"type":"text","props":{"size":10,"width":20,"padding":5},"inline":["abcde"]}}"#,
        None,
        None,
    );
    assert_eq!(b.width, 30.0);
    assert_eq!(b.height, 40.0);
}
