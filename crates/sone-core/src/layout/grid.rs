use taffy::geometry::Size;
use taffy::style::AvailableSpace;

use crate::ir::{Dim, GridTrack};
use crate::paint::TextEngine;

use super::engine::{layout_subtree, BoxLayout, Flat, LayoutState};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TrackKind {
    Fixed,
    Auto,
    Fr,
}

#[derive(Debug, Clone, Copy)]
struct Track {
    kind: TrackKind,
    value: f32,
}

impl From<GridTrack> for Track {
    fn from(t: GridTrack) -> Track {
        match t {
            GridTrack::Fixed(v) => Track {
                kind: TrackKind::Fixed,
                value: v,
            },
            GridTrack::Auto => Track {
                kind: TrackKind::Auto,
                value: 0.0,
            },
            GridTrack::Fr(v) => Track {
                kind: TrackKind::Fr,
                value: v,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridChild {
    pub index: usize,
    pub x: f32,
    pub y: f32,
    pub layout: BoxLayout,
}

#[derive(Debug, Clone, Default)]
pub struct GridResolved {
    pub width: f32,
    pub height: f32,
    pub column_widths: Vec<f32>,
    pub row_heights: Vec<f32>,
    pub children: Vec<GridChild>,
}

struct Placement {
    index: usize,
    column_start: usize,
    column_span: usize,
    row_start: usize,
    row_span: usize,
}

fn normalize(tracks: Option<&Vec<GridTrack>>, fallback: &[GridTrack]) -> Vec<Track> {
    let values: &[GridTrack] = match tracks {
        Some(t) if !t.is_empty() => t,
        _ => fallback,
    };
    values.iter().copied().map(Track::from).collect()
}

fn expand(tracks: &mut Vec<Track>, count: usize, auto: &[Track]) {
    let source: Vec<Track> = if auto.is_empty() {
        vec![Track {
            kind: TrackKind::Auto,
            value: 0.0,
        }]
    } else {
        auto.to_vec()
    };
    while tracks.len() < count {
        tracks.push(source[tracks.len() % source.len()]);
    }
}

fn sum_span(sizes: &[f32], start: usize, span: usize, gap: f32) -> f32 {
    let mut total = 0.0;
    for i in 0..span {
        total += sizes.get(start + i).copied().unwrap_or(0.0);
    }
    if span > 1 {
        total += gap * (span - 1) as f32;
    }
    total
}

fn track_offsets(sizes: &[f32], gap: f32) -> Vec<f32> {
    let mut offsets = Vec::with_capacity(sizes.len());
    let mut cursor = 0.0;
    for s in sizes {
        offsets.push(cursor);
        cursor += s + gap;
    }
    offsets
}

fn distribute_deficit(
    sizes: &mut [f32],
    tracks: &[Track],
    start: usize,
    span: usize,
    deficit: f32,
) {
    if deficit <= 0.0 {
        return;
    }
    let adjustable: Vec<usize> = (0..span)
        .map(|i| start + i)
        .filter(|i| {
            tracks
                .get(*i)
                .map(|t| t.kind != TrackKind::Fixed)
                .unwrap_or(false)
        })
        .collect();
    if adjustable.is_empty() {
        return;
    }
    let share = deficit / adjustable.len() as f32;
    for i in adjustable {
        sizes[i] += share;
    }
}

fn place(
    flat: &Flat<'_>,
    index: usize,
    columns: &mut Vec<Track>,
    rows: &mut Vec<Track>,
    auto_columns: &[Track],
    auto_rows: &[Track],
) -> Vec<Placement> {
    let mut occupied: Vec<Vec<bool>> = Vec::new();
    let mut placements = Vec::new();

    macro_rules! ensure_rows {
        ($count:expr, $cols:expr) => {
            while occupied.len() < $count {
                occupied.push(vec![false; $cols]);
            }
        };
    }

    for &child_index in &flat.children[index] {
        let props = &flat.nodes[child_index].props;
        let column_span = props.grid_column_span.unwrap_or(1).max(1) as usize;
        let row_span = props.grid_row_span.unwrap_or(1).max(1) as usize;
        let column_start_prop = props.grid_column_start;
        let row_start_prop = props.grid_row_start;

        if let Some(c) = column_start_prop {
            let need = c as usize - 1 + column_span;
            expand(columns, need, auto_columns);
        }
        if let Some(r) = row_start_prop {
            expand(rows, r as usize - 1 + row_span, auto_rows);
        }

        let mut row_index = row_start_prop.map(|r| r as usize - 1).unwrap_or(0);
        let mut column_index = column_start_prop.map(|c| c as usize - 1).unwrap_or(0);

        let can_place = |occupied: &mut Vec<Vec<bool>>,
                         columns: &mut Vec<Track>,
                         r0: usize,
                         c0: usize,
                         rs: usize,
                         cs: usize| {
            expand(columns, c0 + cs, auto_columns);
            let cols = columns.len();
            while occupied.len() < r0 + rs {
                occupied.push(vec![false; cols]);
            }
            for row in occupied.iter_mut() {
                while row.len() < cols {
                    row.push(false);
                }
            }
            for row in r0..r0 + rs {
                for col in c0..c0 + cs {
                    if occupied[row][col] {
                        return false;
                    }
                }
            }
            true
        };

        match (row_start_prop, column_start_prop) {
            (Some(_), Some(_)) => {}
            (Some(_), None) => {
                while !can_place(
                    &mut occupied,
                    columns,
                    row_index,
                    column_index,
                    row_span,
                    column_span,
                ) {
                    column_index += 1;
                }
            }
            (None, Some(_)) => {
                while !can_place(
                    &mut occupied,
                    columns,
                    row_index,
                    column_index,
                    row_span,
                    column_span,
                ) {
                    row_index += 1;
                }
            }
            (None, None) => loop {
                let mut placed = false;
                for column in 0..columns.len() {
                    if !can_place(
                        &mut occupied,
                        columns,
                        row_index,
                        column,
                        row_span,
                        column_span,
                    ) {
                        continue;
                    }
                    column_index = column;
                    placed = true;
                    break;
                }
                if placed {
                    break;
                }
                row_index += 1;
            },
        }

        expand(columns, column_index + column_span, auto_columns);
        expand(rows, row_index + row_span, auto_rows);
        let cols = columns.len();
        ensure_rows!(row_index + row_span, cols);
        for row in occupied.iter_mut() {
            while row.len() < cols {
                row.push(false);
            }
        }
        for row in row_index..row_index + row_span {
            for col in column_index..column_index + column_span {
                occupied[row][col] = true;
            }
        }

        placements.push(Placement {
            index: child_index,
            column_start: column_index,
            column_span,
            row_start: row_index,
            row_span,
        });
    }

    placements
}

/// Hand-rolled grid track sizing, ported from `resolveGridLayout`.
pub fn resolve_grid(
    flat: &Flat<'_>,
    index: usize,
    engine: &dyn TextEngine,
    state: &mut LayoutState,
    known: Size<Option<f32>>,
    avail: Size<AvailableSpace>,
    _inset: Size<f32>,
) -> GridResolved {
    let props = &flat.nodes[index].props;
    let gap = props.gap.unwrap_or(0.0);
    let column_gap = props.column_gap.unwrap_or(gap);
    let row_gap = props.row_gap.unwrap_or(gap);

    let mut columns = normalize(props.columns.as_ref(), &[GridTrack::Auto]);
    let mut rows = normalize(props.rows.as_ref(), &[]);
    let auto_columns = normalize(props.auto_columns.as_ref(), &[GridTrack::Auto]);
    let auto_rows = normalize(props.auto_rows.as_ref(), &[GridTrack::Auto]);

    let placements = place(
        flat,
        index,
        &mut columns,
        &mut rows,
        &auto_columns,
        &auto_rows,
    );

    let mut column_widths: Vec<f32> = columns
        .iter()
        .map(|t| {
            if t.kind == TrackKind::Fixed {
                t.value
            } else {
                0.0
            }
        })
        .collect();
    let mut row_heights: Vec<f32> = rows
        .iter()
        .map(|t| {
            if t.kind == TrackKind::Fixed {
                t.value
            } else {
                0.0
            }
        })
        .collect();

    let props = &flat.nodes[index].props;
    let width_definite = constrain(inner(avail.width), props.max_width);
    let height_definite = constrain(inner(avail.height), props.max_height);

    // ── column widths ──
    for p in &placements {
        if p.column_span != 1 {
            continue;
        }
        if columns.get(p.column_start).map(|t| t.kind) == Some(TrackKind::Fixed) {
            continue;
        }
        let m = layout_subtree(flat, p.index, engine, state, None, None, None, None);
        let slot = &mut column_widths[p.column_start];
        *slot = slot.max(m.width);
    }

    let fixed_and_auto: f32 = column_widths
        .iter()
        .enumerate()
        .filter(|(i, _)| columns.get(*i).map(|t| t.kind) != Some(TrackKind::Fr))
        .map(|(_, v)| *v)
        .sum();
    let total_fr: f32 = columns
        .iter()
        .filter(|t| t.kind == TrackKind::Fr)
        .map(|t| t.value)
        .sum();

    if let (Some(w), true) = (width_definite, total_fr > 0.0) {
        let remaining =
            w - fixed_and_auto - column_gap * (column_widths.len().saturating_sub(1)) as f32;
        let unit = remaining.max(0.0) / total_fr;
        for (i, t) in columns.iter().enumerate() {
            if t.kind == TrackKind::Fr {
                column_widths[i] = unit * t.value;
            }
        }
    } else {
        for i in 0..columns.len() {
            if columns[i].kind != TrackKind::Fr {
                continue;
            }
            let mut track_width = 0.0f32;
            for p in placements
                .iter()
                .filter(|p| p.column_span == 1 && p.column_start == i)
            {
                let m = layout_subtree(flat, p.index, engine, state, None, None, None, None);
                track_width = track_width.max(m.width);
            }
            column_widths[i] = column_widths[i].max(track_width);
        }
    }

    for p in placements.iter().filter(|p| p.column_span > 1) {
        let m = layout_subtree(flat, p.index, engine, state, None, None, None, None);
        let current = sum_span(&column_widths, p.column_start, p.column_span, column_gap);
        distribute_deficit(
            &mut column_widths,
            &columns,
            p.column_start,
            p.column_span,
            m.width - current,
        );
    }

    // ── row heights ──
    for p in &placements {
        if p.row_span != 1 {
            continue;
        }
        if rows.get(p.row_start).map(|t| t.kind) == Some(TrackKind::Fixed) {
            continue;
        }
        let owner_width = sum_span(&column_widths, p.column_start, p.column_span, column_gap);
        let m = super::engine::measure_child(
            flat,
            p.index,
            engine,
            state,
            Some(owner_width),
            None,
            true,
            false,
        );
        let slot = &mut row_heights[p.row_start];
        *slot = slot.max(m.height);
    }

    let fixed_and_auto_h: f32 = row_heights
        .iter()
        .enumerate()
        .filter(|(i, _)| rows.get(*i).map(|t| t.kind) != Some(TrackKind::Fr))
        .map(|(_, v)| *v)
        .sum();
    let total_fr_h: f32 = rows
        .iter()
        .filter(|t| t.kind == TrackKind::Fr)
        .map(|t| t.value)
        .sum();

    if let (Some(h), true) = (height_definite, total_fr_h > 0.0) {
        let remaining =
            h - fixed_and_auto_h - row_gap * (row_heights.len().saturating_sub(1)) as f32;
        let unit = remaining.max(0.0) / total_fr_h;
        for (i, t) in rows.iter().enumerate() {
            if t.kind == TrackKind::Fr {
                row_heights[i] = unit * t.value;
            }
        }
    }

    for p in placements.iter().filter(|p| p.row_span > 1) {
        let owner_width = sum_span(&column_widths, p.column_start, p.column_span, column_gap);
        let m = super::engine::measure_child(
            flat,
            p.index,
            engine,
            state,
            Some(owner_width),
            None,
            true,
            false,
        );
        let current = sum_span(&row_heights, p.row_start, p.row_span, row_gap);
        distribute_deficit(
            &mut row_heights,
            &rows,
            p.row_start,
            p.row_span,
            m.height - current,
        );
    }

    let measured_width: f32 = column_widths.iter().sum::<f32>()
        + column_gap * (column_widths.len().saturating_sub(1)) as f32;
    let measured_height: f32 =
        row_heights.iter().sum::<f32>() + row_gap * (row_heights.len().saturating_sub(1)) as f32;

    let _ = known;
    let resolved_width = match width_definite {
        Some(w) if props.width.is_some() => w,
        Some(w) => w.min(measured_width),
        None => measured_width,
    };
    let resolved_height = match height_definite {
        Some(h) if props.height.is_some() => h,
        Some(h) => h.min(measured_height),
        None => measured_height,
    };

    let column_offsets = track_offsets(&column_widths, column_gap);
    let row_offsets = track_offsets(&row_heights, row_gap);

    let mut children = Vec::new();
    for p in &placements {
        let child_width = sum_span(&column_widths, p.column_start, p.column_span, column_gap);
        let child_height = sum_span(&row_heights, p.row_start, p.row_span, row_gap);
        let layout = super::engine::measure_child(
            flat,
            p.index,
            engine,
            state,
            Some(child_width),
            Some(child_height),
            true,
            true,
        );
        children.push(GridChild {
            index: p.index,
            x: column_offsets.get(p.column_start).copied().unwrap_or(0.0),
            y: row_offsets.get(p.row_start).copied().unwrap_or(0.0),
            layout,
        });
    }

    GridResolved {
        width: resolved_width,
        height: resolved_height,
        column_widths,
        row_heights,
        children,
    }
}

fn inner(avail: AvailableSpace) -> Option<f32> {
    match avail {
        AvailableSpace::Definite(v) => Some(v),
        _ => None,
    }
}

fn constrain(value: Option<f32>, max: Option<Dim>) -> Option<f32> {
    let max = match max {
        Some(Dim::Px(v)) => v,
        _ => return value,
    };
    Some(match value {
        Some(v) => v.min(max),
        None => max,
    })
}
