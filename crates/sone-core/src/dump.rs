use serde_json::{json, Value};

use crate::layout::engine::{BoxLayout, Sides};
use crate::style::CompiledNode;

fn sides(s: Sides) -> Value {
    json!({ "top": s.top, "right": s.right, "bottom": s.bottom, "left": s.left })
}

fn node_json(node: &CompiledNode, layout: &BoxLayout, x: f32, y: f32) -> Value {
    let children: Vec<Value> = node
        .children
        .iter()
        .zip(layout.children.iter())
        .map(|(c, l)| node_json(c, l, x + l.x, y + l.y))
        .collect();

    json!({
        "id": node.id,
        "type": format!("{:?}", node.ty).to_lowercase(),
        "tag": node.props.tag,
        "x": x,
        "y": y,
        "left": layout.x,
        "top": layout.y,
        "width": layout.width,
        "height": layout.height,
        "border": sides(layout.border),
        "padding": sides(layout.padding),
        "margin": sides(layout.margin),
        "children": children,
    })
}

/// Computed layout tree, for numeric parity diffing against the TS engine.
pub fn layout_json(root: &CompiledNode, layout: &BoxLayout) -> Value {
    node_json(root, layout, layout.x, layout.y)
}
