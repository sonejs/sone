//! The builder, which touches no rendering at all.

use serde_json::json;
use sone::prelude::*;

fn props(node: impl IntoNode) -> serde_json::Value {
    let value = serde_json::to_value(node.into_node()).unwrap();
    value.get("props").cloned().unwrap_or(json!({}))
}

#[test]
fn chaining_keeps_the_concrete_type() {
    // If the macros stopped expanding per type this would not compile.
    let node: Column = column().gap(20).padding(20).bg("khaki").corner_radius(8);
    assert_eq!(props(node)["gap"], json!(20.0));
}

#[test]
fn numbers_do_not_have_to_be_floats() {
    // `Num` is the whole reason `.gap(20)` works next to `.gap(20.0)`.
    assert_eq!(props(column().gap(20))["gap"], json!(20.0));
    assert_eq!(props(column().gap(20.5))["gap"], json!(20.5));
}

#[test]
fn dim_takes_a_number_or_a_variant() {
    let node = column()
        .width(100)
        .min_width(Dim::Percent(50.0))
        .max_width(Dim::Auto);
    let props = props(node);
    assert_eq!(props["width"], json!(100.0));
    assert_eq!(props["minWidth"], json!("50%"));
    assert_eq!(props["maxWidth"], json!("auto"));
}

#[test]
fn size_and_square() {
    assert_eq!(props(column().square(50))["height"], json!(50.0));
    assert_eq!(props(column().size(420, 300))["width"], json!(420.0));
}

#[test]
fn padding_is_one_value_or_four() {
    assert_eq!(props(column().padding(12))["padding"], json!(12.0));
    let each = props(column().padding_each(1, 2, 3, 4));
    assert_eq!(each["paddingTop"], json!(1.0));
    assert_eq!(each["paddingLeft"], json!(4.0));
}

#[test]
fn keywords_are_enums() {
    let node = row()
        .justify_content(JustifyContent::SpaceBetween)
        .align_items(AlignItems::Center);
    let props = props(node);
    assert_eq!(props["justifyContent"], json!("space-between"));
    assert_eq!(props["alignItems"], json!("center"));
}

#[test]
fn backgrounds_accumulate_and_take_a_photo() {
    let node = column().bg("red").bg(photo("wall.png").into_node());
    let layers = props(node)["background"].clone();
    assert_eq!(layers[0], json!("red"));
    assert_eq!(layers[1]["type"], json!("photo"));
}

#[test]
fn corner_radius_takes_one_or_four() {
    assert_eq!(
        props(column().corner_radius(8))["cornerRadius"],
        json!([8.0])
    );
    assert_eq!(
        props(column().corner_radius([1, 2, 3, 4]))["cornerRadius"],
        json!([1.0, 2.0, 3.0, 4.0])
    );
}

#[test]
fn filters_keep_the_order_they_were_added_in() {
    assert_eq!(
        props(column().blur(4).grayscale(0.5))["filters"],
        json!(["blur(4px)", "grayscale(0.5)"])
    );
}

#[test]
fn text_size_is_the_font_size_not_the_box_size() {
    // Distinct types mean this is a compile-time fact, not a resolution rule:
    // `Text::size` and `Column::size` are simply different methods.
    let props = props(text("Hello").size(28));
    assert_eq!(props["size"], json!(28.0));
    assert!(props.get("width").is_none());
}

#[test]
fn a_text_is_still_a_box() {
    assert_eq!(props(text("Hello").width(120))["width"], json!(120.0));
}

#[test]
fn text_takes_content_and_spans() {
    let node = text("Hello ")
        .font_family("Inter")
        .span(span("world").weight("bold").color("salmon"));
    let value = serde_json::to_value(node.into_node()).unwrap();
    assert_eq!(value["inline"][0], json!("Hello "));
    assert_eq!(value["inline"][1]["type"], json!("span"));
    assert_eq!(value["inline"][1]["props"]["weight"], json!("bold"));
}

#[test]
fn a_decoration_colour_can_be_explicitly_null() {
    let props = props(text("x").underline(1).underline_color(None::<String>));
    assert!(props.as_object().unwrap().contains_key("underlineColor"));
    assert_eq!(props["underlineColor"], json!(null));
}

#[test]
fn weight_takes_a_keyword_or_a_number() {
    assert_eq!(props(text("x").weight("bold"))["weight"], json!("bold"));
    assert_eq!(props(text("x").weight(700))["weight"], json!(700.0));
}

#[test]
fn children_come_from_an_iterator() {
    let rows = ["a", "b", "c"];
    let node = table().children(
        rows.iter()
            .map(|cell| table_row().child(table_cell().child(text(*cell)))),
    );
    let value = serde_json::to_value(node.into_node()).unwrap();
    assert_eq!(value["children"].as_array().unwrap().len(), 3);
}

#[test]
fn maybe_child_drops_a_none() {
    let hidden: Option<Column> = None;
    let node = column().child(column()).maybe_child(hidden);
    let value = serde_json::to_value(node.into_node()).unwrap();
    assert_eq!(value["children"].as_array().unwrap().len(), 1);
}

#[test]
fn grid_tracks() {
    let node = grid().columns([GridTrack::Fr(1.0), GridTrack::Auto, GridTrack::Fixed(120.0)]);
    assert_eq!(props(node)["columns"], json!(["1fr", "auto", 120.0]));
}

#[test]
fn page_break_is_a_zero_height_column() {
    let props = props(page_break());
    assert_eq!(props["height"], json!(0.0));
    assert_eq!(props["pageBreak"], json!("before"));
}

#[test]
fn photo_bytes_become_a_data_url() {
    let src = props(photo_bytes(b"hello world"))["src"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        src.starts_with("data:application/octet-stream;base64,"),
        "{src}"
    );
    assert!(src.ends_with("aGVsbG8gd29ybGQ="), "{src}");
}

#[test]
fn raw_props_are_the_escape_hatch() {
    let node = column().props(|props| props.tag = Some("hatch".into()));
    assert_eq!(props(node)["tag"], json!("hatch"));
}

#[test]
fn the_document_carries_the_schema_version() {
    let json: serde_json::Value = serde_json::from_str(&sone::render(column()).to_json()).unwrap();
    assert_eq!(json["sone"], json!(1));
    assert_eq!(json["root"]["type"], json!("column"));
}

#[test]
fn pagination_tokens_are_passed_through_untouched() {
    let json = sone::render(column())
        .page_height(800)
        .header(text("Page {pageNumber}"))
        .to_json();
    assert!(json.contains("{pageNumber}"), "{json}");
}
