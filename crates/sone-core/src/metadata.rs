use serde_json::{json, Value};

use crate::draw::text::place_runs;
use crate::layout::engine::{BoxLayout, LayoutState, Sides};
use crate::style::{CompiledNode, Content};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Granularity {
    Node,
    Line,
    Word,
}

impl Granularity {
    pub fn parse(s: &str) -> Granularity {
        match s {
            "line" => Granularity::Line,
            "word" => Granularity::Word,
            _ => Granularity::Node,
        }
    }
}

fn sides(s: Sides) -> Value {
    json!({ "top": s.top, "right": s.right, "bottom": s.bottom, "left": s.left })
}

/// Boxes for the text inside one node, at the requested granularity.
fn text_segments(
    node: &CompiledNode,
    layout: &BoxLayout,
    state: &LayoutState,
    granularity: Granularity,
) -> Vec<Value> {
    let Some(content) = node.text() else {
        return Vec::new();
    };
    let Some(text_layout) = state.text.get(&layout.index) else {
        return Vec::new();
    };

    let runs = place_runs(
        content,
        &text_layout.paragraphs,
        layout,
        0.0,
        0.0,
        content.block.orientation,
        true,
    );

    let mut out = Vec::new();
    for run in runs {
        if run.segment.text.is_empty() {
            continue;
        }
        let base = json!({
            "text": run.segment.text,
            "x": run.x,
            "y": run.top(),
            "width": run.width,
            "height": run.segment.height,
            "tag": run.segment.style.tag,
        });
        match granularity {
            Granularity::Word => {
                // Split the run into whitespace-delimited words, distributing
                // the run width by character count.
                let total: f32 = run.segment.text.chars().count().max(1) as f32;
                let mut cursor = 0usize;
                for word in run.segment.text.split_inclusive(char::is_whitespace) {
                    let trimmed = word.trim_end();
                    if !trimmed.is_empty() {
                        let start = cursor as f32;
                        let len = trimmed.chars().count() as f32;
                        out.push(json!({
                            "text": trimmed,
                            "x": run.x + run.width * (start / total),
                            "y": run.top(),
                            "width": run.width * (len / total),
                            "height": run.segment.height,
                            "tag": run.segment.style.tag,
                        }));
                    }
                    cursor += word.chars().count();
                }
            }
            _ => out.push(base),
        }
    }
    out
}

fn node_json(
    node: &CompiledNode,
    layout: &BoxLayout,
    state: &LayoutState,
    x: f32,
    y: f32,
    granularity: Granularity,
) -> Value {
    let mut value = json!({
        "tag": node.props.tag,
        "type": format!("{:?}", node.ty).to_lowercase(),
        "x": x,
        "y": y,
        "width": layout.width,
        "height": layout.height,
        "position": { "top": layout.y, "left": layout.x, "bottom": 0.0, "right": 0.0 },
        "padding": sides(layout.padding),
        "margin": sides(layout.margin),
        "border": sides(layout.border),
    });

    if matches!(node.content, Content::Text(_)) {
        let segments: Vec<Value> = text_segments(node, layout, state, granularity)
            .into_iter()
            .map(|mut s| {
                if let Some(o) = s.as_object_mut() {
                    let sx = o.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let sy = o.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    o.insert("x".into(), json!(x + sx));
                    o.insert("y".into(), json!(y + sy));
                }
                s
            })
            .collect();
        value["segments"] = Value::Array(segments);
        value["children"] = Value::Array(Vec::new());
        return value;
    }

    let children: Vec<Value> = node
        .children
        .iter()
        .zip(layout.children.iter())
        .map(|(c, l)| node_json(c, l, state, x + l.x, y + l.y, granularity))
        .collect();
    value["children"] = Value::Array(children);
    value
}

/// Dataset-style metadata: one box per node, plus text boxes.
pub fn build(
    root: &CompiledNode,
    layout: &BoxLayout,
    state: &LayoutState,
    granularity: &str,
) -> Value {
    node_json(
        root,
        layout,
        state,
        layout.x,
        layout.y,
        Granularity::parse(granularity),
    )
}
