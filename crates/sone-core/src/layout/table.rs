use taffy::prelude::*;
use taffy::style::Position;

use crate::paint::TextEngine;

use super::engine::{layout_subtree, Flat, LayoutState};

#[derive(Debug, Clone)]
pub enum CellEntry {
    Real {
        index: usize,
        dom_index: usize,
        colspan: usize,
        rowspan: usize,
    },
    /// Position claimed by a neighbouring cell's span.
    Phantom {
        source_row: usize,
        source_col: usize,
    },
}

#[derive(Debug, Clone, Default)]
pub struct TableInfo {
    pub grid: Vec<Vec<Option<CellEntry>>>,
    pub col_widths: Vec<f32>,
    pub row_heights: Vec<f32>,
}

/// Expand colspan/rowspan into an occupancy grid, mirroring `buildCellGrid`.
pub fn build_cell_grid(flat: &Flat<'_>, index: usize) -> (Vec<Vec<Option<CellEntry>>>, usize) {
    let rows = &flat.children[index];
    let num_rows = rows.len();
    let mut grid: Vec<Vec<Option<CellEntry>>> = vec![Vec::new(); num_rows];
    let mut occupied: Vec<Vec<Option<(usize, usize)>>> = vec![Vec::new(); num_rows];
    let mut num_cols = 0usize;

    let put = |v: &mut Vec<Option<CellEntry>>, at: usize, value: Option<CellEntry>| {
        while v.len() <= at {
            v.push(None);
        }
        v[at] = value;
    };

    for r in 0..num_rows {
        let cells = &flat.children[rows[r]];
        let mut dom_index = 0usize;
        let mut c = 0usize;

        loop {
            let claimed = occupied[r].get(c).copied().flatten();
            if let Some((sr, sc)) = claimed {
                put(
                    &mut grid[r],
                    c,
                    Some(CellEntry::Phantom {
                        source_row: sr,
                        source_col: sc,
                    }),
                );
                c += 1;
                continue;
            }
            if dom_index >= cells.len() {
                break;
            }

            let cell_index = cells[dom_index];
            dom_index += 1;

            let props = &flat.nodes[cell_index].props;
            let cs = props.colspan.unwrap_or(1).max(1) as usize;
            let rs = props.rowspan.unwrap_or(1).max(1) as usize;

            put(
                &mut grid[r],
                c,
                Some(CellEntry::Real {
                    index: cell_index,
                    dom_index: dom_index - 1,
                    colspan: cs,
                    rowspan: rs,
                }),
            );

            for rs2 in 0..rs {
                let rr = r + rs2;
                if rr >= num_rows {
                    continue;
                }
                for cs2 in 0..cs {
                    if rs2 == 0 && cs2 == 0 {
                        continue;
                    }
                    let at = c + cs2;
                    while occupied[rr].len() <= at {
                        occupied[rr].push(None);
                    }
                    occupied[rr][at] = Some((r, c));
                }
            }

            num_cols = num_cols.max(c + cs);
            c += cs;
        }

        let mut cc = c;
        while let Some(Some((sr, sc))) = occupied[r].get(cc).copied() {
            put(
                &mut grid[r],
                cc,
                Some(CellEntry::Phantom {
                    source_row: sr,
                    source_col: sc,
                }),
            );
            num_cols = num_cols.max(cc + 1);
            cc += 1;
        }
    }

    (grid, num_cols)
}

fn set_min_width(tree: &mut TaffyTree<usize>, id: NodeId, w: f32) {
    let mut s = tree.style(id).unwrap().clone();
    s.min_size.width = length(w);
    tree.set_style(id, s).unwrap();
}

fn set_min_height(tree: &mut TaffyTree<usize>, id: NodeId, h: f32) {
    let mut s = tree.style(id).unwrap().clone();
    s.min_size.height = length(h);
    tree.set_style(id, s).unwrap();
}

/// The four-pass table sizing from `createLayoutNode`: measure natural sizes,
/// widen colspan cells, equalize rows, then float rowspan cells absolutely.
pub fn apply_table_layout(
    tree: &mut TaffyTree<usize>,
    flat: &Flat<'_>,
    index: usize,
    row_ids: &[NodeId],
    engine: &dyn TextEngine,
    state: &mut LayoutState,
) {
    let (grid, num_cols) = build_cell_grid(flat, index);
    let num_rows = flat.children[index].len();
    let mut col_widths = vec![0.0f32; num_cols];
    let mut row_heights = vec![0.0f32; num_rows];

    let mut natural: Vec<Vec<(f32, f32)>> = vec![Vec::new(); num_rows];

    for r in 0..num_rows {
        let mut max_h = 0.0f32;
        for c in 0..grid[r].len() {
            let Some(CellEntry::Real {
                index: cell,
                colspan,
                rowspan,
                ..
            }) = grid[r][c]
            else {
                natural[r].push((0.0, 0.0));
                continue;
            };
            let m = layout_subtree(flat, cell, engine, state, None, None, None, None);
            natural[r].push((m.width, m.height));
            if colspan == 1 && m.width > col_widths[c] {
                col_widths[c] = m.width;
            }
            if rowspan == 1 && m.height > max_h {
                max_h = m.height;
            }
        }
        row_heights[r] = max_h;
    }

    for r in 0..num_rows {
        for c in 0..grid[r].len() {
            let Some(CellEntry::Real { colspan, .. }) = grid[r][c] else {
                continue;
            };
            if colspan <= 1 {
                continue;
            }
            let w = natural[r][c].0;
            let allocated: f32 = col_widths[c..(c + colspan).min(num_cols)].iter().sum();
            if w > allocated {
                let excess = (w - allocated) / colspan as f32;
                for slot in col_widths.iter_mut().skip(c).take(colspan) {
                    *slot += excess;
                }
            }
        }
    }

    for r in 0..num_rows {
        let Some(&row_id) = row_ids.get(r) else {
            continue;
        };
        let mut accumulated = 0.0f32;

        for c in 0..grid[r].len() {
            match grid[r][c] {
                Some(CellEntry::Phantom { source_row, .. }) => {
                    if source_row != r {
                        accumulated += col_widths.get(c).copied().unwrap_or(0.0);
                    }
                }
                Some(CellEntry::Real {
                    dom_index,
                    colspan,
                    rowspan,
                    ..
                }) => {
                    let cell_id = tree.child_at_index(row_id, dom_index).unwrap();
                    if accumulated > 0.0 {
                        let mut s = tree.style(cell_id).unwrap().clone();
                        let existing = existing_left(&s);
                        s.margin.left = length(existing + accumulated);
                        tree.set_style(cell_id, s).unwrap();
                        accumulated = 0.0;
                    }
                    let span_w: f32 = col_widths[c..(c + colspan).min(num_cols)].iter().sum();
                    set_min_width(tree, cell_id, span_w);
                    if rowspan > 1 {
                        accumulated += span_w;
                    }
                }
                None => {}
            }
        }
    }

    for r in 0..num_rows {
        let Some(&row_id) = row_ids.get(r) else {
            continue;
        };
        for c in 0..grid[r].len() {
            let Some(CellEntry::Real {
                dom_index, rowspan, ..
            }) = grid[r][c]
            else {
                continue;
            };
            if rowspan > 1 {
                continue;
            }
            let cell_id = tree.child_at_index(row_id, dom_index).unwrap();
            set_min_height(tree, cell_id, row_heights[r]);
        }
    }

    // Rowspan cells float out of flow so they cannot inflate their host row.
    for r in 0..num_rows {
        let Some(&row_id) = row_ids.get(r) else {
            continue;
        };
        for c in 0..grid[r].len() {
            let Some(CellEntry::Real {
                dom_index,
                colspan,
                rowspan,
                ..
            }) = grid[r][c]
            else {
                continue;
            };
            if rowspan <= 1 {
                continue;
            }
            let cell_id = tree.child_at_index(row_id, dom_index).unwrap();
            let offset_x: f32 = col_widths[..c.min(num_cols)].iter().sum();
            let span_w: f32 = col_widths[c..(c + colspan).min(num_cols)].iter().sum();
            let span_h: f32 = row_heights[r..(r + rowspan).min(num_rows)].iter().sum();

            let mut s = tree.style(cell_id).unwrap().clone();
            s.position = Position::Absolute;
            s.inset.left = length(offset_x);
            s.inset.top = length(0.0f32);
            s.size.width = length(span_w);
            s.size.height = length(span_h);
            tree.set_style(cell_id, s).unwrap();
        }
    }

    state.table.insert(
        index,
        TableInfo {
            grid,
            col_widths,
            row_heights,
        },
    );
}

/// Only a pixel margin carries over, matching the TS `typeof === "number"` guard.
fn existing_left(style: &Style) -> f32 {
    let raw = style.margin.left.into_raw();
    if raw.tag() == taffy::style::CompactLength::LENGTH_TAG {
        raw.value()
    } else {
        0.0
    }
}
