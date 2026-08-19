use crate::ir::{self, PageBreak};
use crate::layout::engine::{BoxLayout, LayoutState};
use crate::style::{CompiledNode, Content, Inline};

/// Break points closer together than this collapse into one, so a text-line
/// break landing just before an explicit `PageBreak()` cannot make a hairline page.
const COLLAPSE_WITHIN: f32 = 20.0;

/// Node types treated as indivisible when a page boundary crosses them.
fn is_leaf(ty: ir::NodeType) -> bool {
    matches!(
        ty,
        ir::NodeType::Text | ir::NodeType::Photo | ir::NodeType::Path
    )
}

struct Walker<'a> {
    state: &'a LayoutState,
    page_height: f32,
    current_page_end: f32,
    breaks: Vec<f32>,
}

impl Walker<'_> {
    fn push(&mut self, at: f32) {
        let last = self.breaks.last().copied().unwrap_or(0.0);
        if at > last {
            self.breaks.push(at);
            self.current_page_end = at + self.page_height;
        }
    }

    fn advance_to(&mut self, target: f32) {
        while self.current_page_end < target {
            let end = self.current_page_end;
            self.breaks.push(end);
            self.current_page_end += self.page_height;
        }
    }

    fn walk(&mut self, node: &CompiledNode, layout: &BoxLayout, abs_y: f32) {
        for (child, child_layout) in node.children.iter().zip(layout.children.iter()) {
            let top = abs_y + child_layout.y;
            let bottom = top + child_layout.height;
            let page_break = child.props.page_break;

            if page_break == Some(PageBreak::Before) && top > 0.0 {
                self.advance_to(top);
                if top < self.current_page_end {
                    self.push(top);
                }
            }

            if bottom <= self.current_page_end {
                if page_break == Some(PageBreak::After) {
                    self.push(bottom);
                }
                continue;
            }

            let atomic = page_break == Some(PageBreak::Avoid) || is_leaf(child.ty);
            let height = bottom - top;

            if matches!(child.content, Content::Text(_)) {
                self.split_text(child, child_layout, top);
            } else if atomic {
                if top < self.current_page_end {
                    if height <= self.page_height {
                        let end = self.current_page_end;
                        self.breaks.push(end);
                        self.current_page_end += self.page_height;
                    }
                } else {
                    while top >= self.current_page_end {
                        let end = self.current_page_end;
                        self.breaks.push(end);
                        self.current_page_end += self.page_height;
                    }
                }
                while bottom > self.current_page_end {
                    let end = self.current_page_end;
                    self.breaks.push(end);
                    self.current_page_end += self.page_height;
                }
            } else {
                self.walk(child, child_layout, top);
            }

            if page_break == Some(PageBreak::After) {
                let end = self.current_page_end;
                self.breaks.push(end);
                self.current_page_end += self.page_height;
            }
        }
    }

    /// Split a paragraph at line boundaries so no line is clipped in half.
    fn split_text(&mut self, node: &CompiledNode, layout: &BoxLayout, top: f32) {
        let blocks = self.state.text.get(&layout.index).map(|t| &t.paragraphs);
        let Some(blocks) = blocks.filter(|b| !b.is_empty()) else {
            let bottom = top + layout.height;
            if top < self.current_page_end && layout.height <= self.page_height {
                let end = self.current_page_end;
                self.breaks.push(end);
                self.current_page_end += self.page_height;
            } else {
                while top >= self.current_page_end {
                    let end = self.current_page_end;
                    self.breaks.push(end);
                    self.current_page_end += self.page_height;
                }
            }
            while bottom > self.current_page_end {
                let end = self.current_page_end;
                self.breaks.push(end);
                self.current_page_end += self.page_height;
            }
            let _ = node;
            return;
        };

        let inset_top = layout.padding.top + layout.border.top;
        let mut paragraph_y = 0.0f32;
        for paragraph in blocks {
            let mut line_y = paragraph_y + paragraph.offset_y;
            for line in &paragraph.lines {
                let line_bottom = top + inset_top + line_y + line.height;
                if line_bottom > self.current_page_end {
                    let line_top = (top + inset_top + line_y).floor();
                    self.push(line_top);
                    while line_bottom > self.current_page_end {
                        let end = self.current_page_end;
                        self.breaks.push(end);
                        self.current_page_end += self.page_height;
                    }
                }
                line_y += line.height;
            }
            paragraph_y += paragraph.height;
        }
    }
}

/// Absolute y offsets where the content should be cut into pages.
pub fn compute_page_breaks(
    root: &CompiledNode,
    layout: &BoxLayout,
    state: &LayoutState,
    page_height: f32,
) -> Vec<f32> {
    if page_height <= 0.0 {
        return Vec::new();
    }
    let mut walker = Walker {
        state,
        page_height,
        current_page_end: page_height,
        breaks: Vec::new(),
    };
    walker.walk(root, layout, 0.0);

    let mut sorted = walker.breaks;
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sorted.dedup();

    let mut collapsed: Vec<f32> = Vec::new();
    for b in sorted {
        if collapsed
            .last()
            .map(|last| b - last > COLLAPSE_WITHIN)
            .unwrap_or(true)
        {
            collapsed.push(b);
        }
    }
    collapsed
}

/// Replace `{pageNumber}` / `{totalPages}` in every text run of a header or footer.
pub fn substitute_page_tokens(
    node: &CompiledNode,
    page_number: usize,
    total_pages: usize,
) -> CompiledNode {
    let mut out = node.clone();
    let page = page_number.to_string();
    let total = total_pages.to_string();

    fn visit(node: &mut CompiledNode, page: &str, total: &str) {
        if let Content::Text(content) = &mut node.content {
            for inline in &mut content.inlines {
                match inline {
                    Inline::Text(t) => {
                        *t = t
                            .replace("{pageNumber}", page)
                            .replace("{totalPages}", total)
                    }
                    Inline::Span { text, .. } => {
                        *text = text
                            .replace("{pageNumber}", page)
                            .replace("{totalPages}", total)
                    }
                }
            }
        }
        for child in &mut node.children {
            visit(child, page, total);
        }
    }

    visit(&mut out, &page, &total);
    out
}

/// True when any text run still carries a page-number token.
pub fn has_page_tokens(node: &CompiledNode) -> bool {
    if let Content::Text(content) = &node.content {
        if content
            .inlines
            .iter()
            .any(|i| i.text().contains("{pageNumber}") || i.text().contains("{totalPages}"))
        {
            return true;
        }
    }
    node.children.iter().any(has_page_tokens)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Margins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl From<Option<ir::MarginSpec>> for Margins {
    fn from(value: Option<ir::MarginSpec>) -> Self {
        match value {
            Some(spec) => Margins {
                top: spec.0.top,
                right: spec.0.right,
                bottom: spec.0.bottom,
                left: spec.0.left,
            },
            None => Margins::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::{compile, CompileCtx, NullAssets};
    use crate::ir::Document;
    use crate::layout::engine::layout;
    use crate::testing::FixedMetricsEngine;

    fn breaks(json: &str, page_height: f32) -> Vec<f32> {
        let doc = Document::from_json(json).unwrap();
        let assets = NullAssets;
        let mut ctx = CompileCtx::new(&assets);
        let root = compile(&doc.root, &mut ctx).unwrap().unwrap();
        let engine = FixedMetricsEngine::default();
        let (tree, state) = layout(&root, &engine, doc.config.width, None);
        compute_page_breaks(&root, &tree, &state, page_height)
    }

    #[test]
    fn atomic_blocks_break_between_children() {
        // Only leaf nodes and `pageBreak: "avoid"` blocks are indivisible; an
        // empty container is walked into and contributes no break of its own.
        let b = breaks(
            r#"{"sone":1,"root":{"type":"column","children":[
                 {"type":"column","props":{"height":80,"pageBreak":"avoid"}},
                 {"type":"column","props":{"height":80,"pageBreak":"avoid"}},
                 {"type":"column","props":{"height":80,"pageBreak":"avoid"}}]}}"#,
            100.0,
        );
        assert_eq!(b, vec![100.0, 200.0]);
    }

    #[test]
    fn empty_containers_contribute_no_breaks() {
        let b = breaks(
            r#"{"sone":1,"root":{"type":"column","children":[
                 {"type":"column","props":{"height":80}},
                 {"type":"column","props":{"height":80}}]}}"#,
            100.0,
        );
        assert!(b.is_empty(), "{b:?}");
    }

    #[test]
    fn explicit_break_before_starts_a_page() {
        let b = breaks(
            r#"{"sone":1,"root":{"type":"column","children":[
                 {"type":"column","props":{"height":30}},
                 {"type":"column","props":{"height":30,"pageBreak":"before"}}]}}"#,
            1000.0,
        );
        assert_eq!(b, vec![30.0]);
    }

    #[test]
    fn nearby_breaks_collapse() {
        let b = breaks(
            r#"{"sone":1,"root":{"type":"column","children":[
                 {"type":"column","props":{"height":95}},
                 {"type":"column","props":{"height":5,"pageBreak":"before"}},
                 {"type":"column","props":{"height":50}}]}}"#,
            100.0,
        );
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn text_splits_at_line_boundaries() {
        // 6 lines of 10px; a 25px page must break at line boundaries only.
        let b = breaks(
            r#"{"sone":1,"root":{"type":"column","props":{"width":30},"children":[
                 {"type":"text","props":{"size":10},"inline":["aa bb cc dd ee ff"]}]}}"#,
            25.0,
        );
        assert!(b.iter().all(|v| v % 10.0 == 0.0), "{b:?}");
    }

    #[test]
    fn page_tokens_are_substituted() {
        let doc = Document::from_json(
            r#"{"sone":1,"root":{"type":"text","inline":["{pageNumber} of {totalPages}"]}}"#,
        )
        .unwrap();
        let assets = NullAssets;
        let mut ctx = CompileCtx::new(&assets);
        let root = compile(&doc.root, &mut ctx).unwrap().unwrap();
        assert!(has_page_tokens(&root));
        let out = substitute_page_tokens(&root, 3, 7);
        assert_eq!(out.text().unwrap().inlines[0].text(), "3 of 7");
        assert!(!has_page_tokens(&out));
    }
}
