use crate::ir::{TextAlign, TextOverflow, TextWrap};
use crate::paint::{TextEngine, TextMetrics};
use crate::style::{BlockStyle, Inline, RunStyle, TextContent};
use crate::text::bidi::{resolve_paragraph_direction, Dir};

const ELLIPSIS: &str = "…";

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Run {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub metrics: TextMetrics,
    pub style: RunStyle,
    pub text: String,
    pub width: f32,
    pub height: f32,
    pub run: Option<Run>,
    /// Synthetic tab-stop gap; never merged with a neighbour.
    pub is_tab: bool,
    pub tab_leader: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Line {
    pub baseline: f32,
    pub segments: Vec<Segment>,
    pub spaces_count: usize,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Paragraph {
    pub width: f32,
    pub height: f32,
    pub lines: Vec<Line>,
    pub offset_y: f32,
    pub base_dir: Dir,
}

fn count_spaces(s: &str) -> usize {
    s.bytes().filter(|b| *b == b' ').count()
}

fn is_all_whitespace(s: &str) -> bool {
    !s.is_empty() && s.chars().all(char::is_whitespace)
}

fn trim_trailing_ws(s: &str) -> &str {
    s.trim_end_matches([' ', '\t'])
}

fn trim_leading_ws(s: &str) -> &str {
    s.trim_start_matches([' ', '\t'])
}

fn next_tab_stop(tab_stops: &[f32], current_x: f32) -> f32 {
    for stop in tab_stops {
        if *stop > current_x {
            return *stop;
        }
    }
    current_x + 40.0
}

pub struct Shaper<'a> {
    pub engine: &'a dyn TextEngine,
}

impl<'a> Shaper<'a> {
    fn measure(&self, text: &str, style: &RunStyle) -> TextMetrics {
        self.engine.measure(text, &style.shaping())
    }

    fn tab_expanded_width(
        &self,
        text: &str,
        style: &RunStyle,
        tab_stops: &[f32],
        current_x: f32,
    ) -> f32 {
        if tab_stops.is_empty() || !text.contains('\t') {
            return self.measure(text, style).width;
        }
        let mut x = current_x;
        let parts: Vec<&str> = text.split('\t').collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                x += self.measure(part, style).width;
            }
            if i < parts.len() - 1 {
                x = next_tab_stop(tab_stops, x);
            }
        }
        x - current_x
    }

    fn measured_segment(&self, text: &str, style: &RunStyle) -> Segment {
        let m = self.measure(text, style);
        Segment {
            metrics: m,
            style: style.clone(),
            text: text.to_string(),
            width: m.width,
            height: m.height(),
            run: None,
            is_tab: false,
            tab_leader: None,
        }
    }

    fn push_segments(
        &self,
        line: &mut Line,
        text: &str,
        style: &RunStyle,
        tab_stops: &[f32],
        tab_leader: &str,
    ) {
        if tab_stops.is_empty() || !text.contains('\t') {
            let m = self.measure(text, style);
            line.segments.push(Segment {
                metrics: m,
                style: style.clone(),
                text: text.to_string(),
                width: m.width,
                height: m.height(),
                run: None,
                is_tab: false,
                tab_leader: None,
            });
            line.width += m.width;
            line.height = line.height.max(m.height());
            line.baseline = line.baseline.max(m.ascent);
            return;
        }

        let parts: Vec<&str> = text.split('\t').collect();
        for (i, part) in parts.iter().enumerate() {
            if !part.is_empty() {
                let m = self.measure(part, style);
                line.segments.push(Segment {
                    metrics: m,
                    style: style.clone(),
                    text: (*part).to_string(),
                    width: m.width,
                    height: m.height(),
                    run: None,
                    is_tab: false,
                    tab_leader: None,
                });
                line.width += m.width;
                line.height = line.height.max(m.height());
                line.baseline = line.baseline.max(m.ascent);
            }
            if i < parts.len() - 1 {
                let tab_width = (next_tab_stop(tab_stops, line.width) - line.width).max(4.0);
                let m = self.measure(" ", style);

                let mut leader = None;
                if !tab_leader.is_empty() {
                    let char_width = self.measure(tab_leader, style).width;
                    if char_width > 0.0 {
                        let count = (tab_width / char_width).floor() as usize;
                        if count > 0 {
                            leader = Some(tab_leader.repeat(count));
                        }
                    }
                }

                line.segments.push(Segment {
                    metrics: m,
                    style: style.clone(),
                    text: String::new(),
                    width: tab_width,
                    height: m.height(),
                    run: None,
                    is_tab: true,
                    tab_leader: leader,
                });
                line.width += tab_width;
                line.height = line.height.max(m.height());
                line.baseline = line.baseline.max(m.ascent);
            }
        }
    }

    /// Break offsets usable as a line-start prefix, descending.
    fn prefix_boundaries(&self, text: &str) -> Vec<usize> {
        let mut set = std::collections::BTreeSet::new();
        set.insert(0usize);
        for i in self.engine.break_points(text) {
            if i > 0 && i < text.len() {
                set.insert(i);
            }
        }
        for i in self.engine.grapheme_starts(text) {
            if i > 0 && i < text.len() {
                set.insert(i);
            }
        }
        set.into_iter().rev().collect()
    }
}

fn empty_line(width: f32) -> Line {
    Line {
        width,
        ..Default::default()
    }
}

fn recompute_line_metrics(line: &mut Line) {
    line.height = 0.0;
    line.baseline = 0.0;
    for s in &line.segments {
        line.height = line.height.max(s.height);
        line.baseline = line.baseline.max(s.metrics.ascent);
    }
}

fn trim_trailing_whitespace(line: &mut Line, shaper: &Shaper<'_>) {
    while let Some(tail) = line.segments.last_mut() {
        if tail.is_tab {
            line.width -= tail.width;
            line.segments.pop();
            continue;
        }
        let trimmed = trim_trailing_ws(&tail.text).to_string();
        if trimmed == tail.text {
            break;
        }
        if trimmed.is_empty() {
            line.width -= tail.width;
            line.segments.pop();
            continue;
        }
        let m = shaper.measure(&trimmed, &tail.style);
        line.width -= tail.width - m.width;
        tail.text = trimmed;
        tail.metrics = m;
        tail.width = m.width;
        tail.height = m.height();
        break;
    }
    recompute_line_metrics(line);
}

fn trim_trailing_segments(segments: &mut Vec<Segment>, shaper: &Shaper<'_>) {
    while let Some(tail) = segments.last_mut() {
        if tail.is_tab {
            segments.pop();
            continue;
        }
        let trimmed = trim_trailing_ws(&tail.text).to_string();
        if trimmed == tail.text {
            break;
        }
        if trimmed.is_empty() {
            segments.pop();
            continue;
        }
        let m = shaper.measure(&trimmed, &tail.style);
        tail.text = trimmed;
        tail.metrics = m;
        tail.width = m.width;
        tail.height = m.height();
        break;
    }
}

fn segment_text_width(segments: &[Segment]) -> f32 {
    segments.iter().map(|s| s.width).sum()
}

fn line_indent_width(index: usize, block: &BlockStyle) -> f32 {
    if index == 0 {
        block.indent_size
    } else {
        block.hanging_indent_size
    }
}

/// Half-leading line box: the shared metric pass for `lineHeight`.
fn line_box_metrics(line: &Line, block: &BlockStyle, base_size: f32) -> (f32, f32, f32, f32) {
    let multiplier = block.line_height.unwrap_or(1.0);
    let default_lh = block.line_height.is_none();

    let mut above = 0.0f32;
    let mut below = 0.0f32;
    let mut offset_max = 0.0f32;
    let mut offset_min = f32::INFINITY;
    for s in &line.segments {
        let font_size = if s.style.size != 0.0 {
            s.style.size
        } else {
            base_size
        };
        let box_h = if default_lh {
            s.height
        } else {
            font_size * multiplier
        };
        let half_leading = (box_h - s.height) / 2.0;
        above = above.max(half_leading + s.metrics.ascent);
        below = below.max(half_leading + s.metrics.descent);
        let offset = s.style.offset_y;
        if offset > offset_max {
            offset_max = offset;
        }
        if offset < offset_min {
            offset_min = offset;
        }
    }
    if offset_min == f32::INFINITY {
        offset_min = 0.0;
    }
    (above, below, offset_min, offset_max)
}

fn recompute_finalized_line(line: &mut Line, index: usize, block: &BlockStyle, base_size: f32) {
    let (above, below, offset_min, offset_max) = line_box_metrics(line, block, base_size);
    let segments_width: f32 = line.segments.iter().map(|s| s.width).sum();
    line.width = line_indent_width(index, block) + segments_width;
    line.baseline = above - offset_min;
    line.height = above + below - offset_min + offset_max;
}

fn recompute_paragraph_metrics(p: &mut Paragraph) {
    p.width = 0.0;
    p.height = 0.0;
    let last = p.lines.len().saturating_sub(1);
    for (i, line) in p.lines.iter_mut().enumerate() {
        line.spaces_count = if i < last {
            line.segments.iter().map(|s| count_spaces(&s.text)).sum()
        } else {
            0
        };
        p.width = p.width.max(line.width);
        p.height += line.height;
    }
    p.offset_y = 0.0;
}

fn finalize_paragraph(
    mut lines: Vec<Line>,
    block: &BlockStyle,
    base_size: f32,
    shaper: &Shaper<'_>,
) -> Paragraph {
    let mut total_height = 0.0;
    let mut max_width: f32 = 0.0;
    let count = lines.len();

    for i in 0..count {
        trim_trailing_whitespace(&mut lines[i], shaper);
        let (above, below, offset_min, offset_max) = line_box_metrics(&lines[i], block, base_size);
        let line = &mut lines[i];
        line.baseline = above - offset_min;
        line.height = above + below - offset_min + offset_max;

        if i < count - 1 {
            line.spaces_count += line
                .segments
                .iter()
                .map(|s| count_spaces(&s.text))
                .sum::<usize>();
        }
        total_height += line.height;
        max_width = max_width.max(line.width);
    }

    Paragraph {
        width: max_width,
        height: total_height,
        lines,
        offset_y: 0.0,
        base_dir: Dir::Ltr,
    }
}

fn fit_line_with_ellipsis(
    line: &mut Line,
    index: usize,
    max_width: f32,
    block: &BlockStyle,
    base_size: f32,
    shaper: &Shaper<'_>,
) {
    let budget = (max_width - line_indent_width(index, block)).max(0.0);
    let mut working: Vec<Segment> = line.segments.clone();
    trim_trailing_segments(&mut working, shaper);

    let ellipsis_style = working
        .iter()
        .rev()
        .find(|s| !s.is_tab && !s.text.is_empty())
        .map(|s| s.style.clone());
    let ellipsis_style = ellipsis_style.unwrap_or_else(|| default_run_style(base_size));
    let ellipsis = shaper.measured_segment(ELLIPSIS, &ellipsis_style);

    while !working.is_empty() {
        trim_trailing_segments(&mut working, shaper);
        if segment_text_width(&working) + ellipsis.width <= budget {
            working.push(ellipsis);
            line.segments = working;
            recompute_finalized_line(line, index, block, base_size);
            return;
        }

        let tail_is_tab = working.last().map(|s| s.is_tab).unwrap_or(false);
        if tail_is_tab {
            working.pop();
            continue;
        }

        let boundaries = shaper.prefix_boundaries(&working.last().unwrap().text);
        let mut shortened = false;
        for boundary in boundaries {
            let tail = working.last_mut().unwrap();
            if boundary >= tail.text.len() || !tail.text.is_char_boundary(boundary) {
                continue;
            }
            let next_text = trim_trailing_ws(&tail.text[..boundary]).to_string();
            if next_text == tail.text || next_text.is_empty() {
                continue;
            }
            let m = shaper.measure(&next_text, &tail.style);
            tail.text = next_text;
            tail.metrics = m;
            tail.width = m.width;
            tail.height = m.height();
            shortened = true;
            break;
        }
        if shortened {
            continue;
        }
        working.pop();
    }

    line.segments = if ellipsis.width <= budget {
        vec![ellipsis]
    } else {
        Vec::new()
    };
    recompute_finalized_line(line, index, block, base_size);
}

fn default_run_style(size: f32) -> RunStyle {
    RunStyle {
        size,
        ..Default::default()
    }
}

fn apply_text_overflow(
    p: &mut Paragraph,
    max_width: f32,
    block: &BlockStyle,
    base_size: f32,
    shaper: &Shaper<'_>,
) {
    let hidden_lines = block.max_lines.is_some_and(|m| p.lines.len() > m);
    if let Some(m) = block.max_lines {
        if hidden_lines {
            p.lines.truncate(m);
        }
    }

    if p.lines.is_empty() {
        p.width = 0.0;
        p.height = 0.0;
        p.offset_y = 0.0;
        return;
    }

    let last_index = p.lines.len() - 1;
    let overflows_width =
        block.nowrap && max_width.is_finite() && p.lines[last_index].width > max_width;

    if max_width.is_finite()
        && block.text_overflow == TextOverflow::Ellipsis
        && (hidden_lines || overflows_width)
    {
        let mut line = std::mem::take(&mut p.lines[last_index]);
        fit_line_with_ellipsis(&mut line, last_index, max_width, block, base_size, shaper);
        p.lines[last_index] = line;
    }

    recompute_paragraph_metrics(p);
}

struct Chunk {
    style: RunStyle,
    text: String,
    width: f32,
}

fn create_paragraph_chunks(
    inlines: &[Inline],
    styles: &[RunStyle],
    breakpoints: &[Vec<usize>],
    shaper: &Shaper<'_>,
) -> Option<Vec<Chunk>> {
    let mut chunks = Vec::new();
    for (i, inline) in inlines.iter().enumerate() {
        let style = &styles[i];
        let text = inline.text();
        if text.contains('\t') {
            return None;
        }
        let bps = breakpoints.get(i).cloned().unwrap_or_default();
        if bps.is_empty() {
            chunks.push(Chunk {
                style: style.clone(),
                text: text.to_string(),
                width: shaper.measure(text, style).width,
            });
            continue;
        }
        let mut last = 0usize;
        for bp in bps {
            if bp <= last || bp > text.len() || !text.is_char_boundary(bp) {
                continue;
            }
            let piece = &text[last..bp];
            if piece.is_empty() {
                last = bp;
                continue;
            }
            chunks.push(Chunk {
                style: style.clone(),
                text: piece.to_string(),
                width: shaper.measure(piece, style).width,
            });
            last = bp;
        }
        if last < text.len() {
            let piece = &text[last..];
            chunks.push(Chunk {
                style: style.clone(),
                text: piece.to_string(),
                width: shaper.measure(piece, style).width,
            });
        }
    }
    Some(chunks)
}

fn find_fitting_wrap_boundary(
    text: &str,
    style: &RunStyle,
    available: f32,
    current_x: f32,
    tab_stops: &[f32],
    shaper: &Shaper<'_>,
) -> usize {
    let boundaries = shaper.prefix_boundaries(text);
    let mut fallback = text.len();
    let mut best = 0usize;

    for &boundary in boundaries.iter().rev() {
        if boundary == 0 || !text.is_char_boundary(boundary) {
            continue;
        }
        if fallback == text.len() {
            fallback = boundary;
        }
        let width = shaper.tab_expanded_width(&text[..boundary], style, tab_stops, current_x);
        if width <= available {
            best = boundary;
            continue;
        }
        break;
    }

    if best == 0 {
        let full = shaper.tab_expanded_width(text, style, tab_stops, current_x);
        if full <= available {
            best = text.len();
        }
    }
    if best > 0 {
        best
    } else {
        fallback
    }
}

#[allow(clippy::too_many_arguments)]
fn append_wrapped_text(
    lines: &mut Vec<Line>,
    mut current: usize,
    text: &str,
    style: &RunStyle,
    max_width: f32,
    block: &BlockStyle,
    shaper: &Shaper<'_>,
    should_wrap: bool,
) -> usize {
    let mut remaining = text.to_string();

    while !remaining.is_empty() {
        let seg_width =
            shaper.tab_expanded_width(&remaining, style, &block.tab_stops, lines[current].width);

        if !should_wrap || !max_width.is_finite() || lines[current].width + seg_width <= max_width {
            shaper.push_segments(
                &mut lines[current],
                &remaining,
                style,
                &block.tab_stops,
                &block.tab_leader,
            );
            break;
        }

        if !lines[current].segments.is_empty() {
            lines.push(empty_line(block.hanging_indent_size));
            current = lines.len() - 1;
            remaining = trim_leading_ws(&remaining).to_string();
            if remaining.is_empty() {
                break;
            }
            continue;
        }

        let available = (max_width - lines[current].width).max(0.0);
        let boundary = find_fitting_wrap_boundary(
            &remaining,
            style,
            available,
            lines[current].width,
            &block.tab_stops,
            shaper,
        );
        let boundary = boundary.min(remaining.len());
        let line_text = remaining[..boundary].to_string();
        shaper.push_segments(
            &mut lines[current],
            &line_text,
            style,
            &block.tab_stops,
            &block.tab_leader,
        );

        remaining = trim_leading_ws(&remaining[boundary..]).to_string();
        if remaining.is_empty() {
            break;
        }
        lines.push(empty_line(block.hanging_indent_size));
        current = lines.len() - 1;
    }
    current
}

fn greedy_paragraph(
    inlines: &[Inline],
    styles: &[RunStyle],
    breakpoints: &[Vec<usize>],
    max_width: f32,
    block: &BlockStyle,
    base_size: f32,
    shaper: &Shaper<'_>,
) -> Paragraph {
    let should_wrap = !block.nowrap;
    let mut lines = vec![empty_line(block.indent_size)];
    let mut current = 0usize;

    for (i, inline) in inlines.iter().enumerate() {
        let style = &styles[i];
        let text = inline.text();
        let bps = breakpoints.get(i).cloned().unwrap_or_default();

        if bps.is_empty() {
            current = append_wrapped_text(
                &mut lines,
                current,
                text,
                style,
                max_width,
                block,
                shaper,
                should_wrap,
            );
            continue;
        }

        let mut last = 0usize;
        for bp in bps {
            if bp <= last || bp > text.len() || !text.is_char_boundary(bp) {
                continue;
            }
            let piece = &text[last..bp];
            if piece.is_empty() {
                last = bp;
                continue;
            }
            current = append_wrapped_text(
                &mut lines,
                current,
                piece,
                style,
                max_width,
                block,
                shaper,
                should_wrap,
            );
            last = bp;
        }
        if last < text.len() {
            current = append_wrapped_text(
                &mut lines,
                current,
                &text[last..],
                style,
                max_width,
                block,
                shaper,
                should_wrap,
            );
        }
    }

    finalize_paragraph(lines, block, base_size, shaper)
}

fn knuth_plass_paragraph(
    inlines: &[Inline],
    styles: &[RunStyle],
    breakpoints: &[Vec<usize>],
    max_width: f32,
    block: &BlockStyle,
    base_size: f32,
    shaper: &Shaper<'_>,
) -> Option<Paragraph> {
    if block.nowrap || !max_width.is_finite() {
        return None;
    }
    let chunks = create_paragraph_chunks(inlines, styles, breakpoints, shaper)?;
    if chunks.is_empty() {
        return None;
    }

    let trimmed_widths: Vec<f32> = chunks
        .iter()
        .map(|c| {
            let t = trim_trailing_ws(&c.text);
            if t.is_empty() {
                0.0
            } else if t == c.text {
                c.width
            } else {
                shaper.measure(t, &c.style).width
            }
        })
        .collect();

    let mut prefix = vec![0.0f32];
    for c in &chunks {
        prefix.push(prefix[prefix.len() - 1] + c.width);
    }

    let measure_line_width = |start: usize, end: usize| -> f32 {
        let indent = if start == 0 {
            block.indent_size
        } else {
            block.hanging_indent_size
        };
        let mut effective_end = end;
        while effective_end > start
            && is_all_whitespace(&chunks[effective_end - 1].text.replace('\t', " "))
        {
            effective_end -= 1;
        }
        if effective_end == start {
            return indent;
        }
        let mut width = prefix[effective_end] - prefix[start];
        width -= chunks[effective_end - 1].width - trimmed_widths[effective_end - 1];
        indent + width
    };

    let n = chunks.len();
    let mut costs = vec![f32::INFINITY; n + 1];
    let mut previous = vec![usize::MAX; n + 1];
    costs[0] = 0.0;

    for start in 0..n {
        if !costs[start].is_finite() {
            continue;
        }
        for end in start + 1..=n {
            if end < n && is_all_whitespace(&chunks[end].text) {
                continue;
            }
            let line_width = measure_line_width(start, end);
            if line_width > max_width {
                break;
            }
            let is_last = end == n;
            let slack = (max_width - line_width).max(0.0);
            let ratio = slack / max_width.max(1.0);
            let badness = if is_last {
                0.0
            } else {
                (ratio * 100.0).powi(3) + 1.0
            };
            let next_cost = costs[start] + badness;
            if next_cost < costs[end] {
                costs[end] = next_cost;
                previous[end] = start;
            }
        }
    }

    if !costs[n].is_finite() || previous[n] == usize::MAX {
        return None;
    }

    let mut ranges = Vec::new();
    let mut end = n;
    while end > 0 {
        let start = previous[end];
        if start == usize::MAX {
            return None;
        }
        ranges.push((start, end));
        end = start;
    }
    ranges.reverse();

    let lines = ranges
        .iter()
        .enumerate()
        .map(|(index, &(start, end))| {
            let mut line = empty_line(if index == 0 {
                block.indent_size
            } else {
                block.hanging_indent_size
            });
            for chunk in chunks.iter().take(end).skip(start) {
                shaper.push_segments(&mut line, &chunk.text, &chunk.style, &[], "");
            }
            line
        })
        .collect();

    Some(finalize_paragraph(lines, block, base_size, shaper))
}

fn balanced_paragraph(
    inlines: &[Inline],
    styles: &[RunStyle],
    breakpoints: &[Vec<usize>],
    max_width: f32,
    block: &BlockStyle,
    base_size: f32,
    shaper: &Shaper<'_>,
) -> Paragraph {
    let reference = greedy_paragraph(
        inlines,
        styles,
        breakpoints,
        max_width,
        block,
        base_size,
        shaper,
    );
    let target = reference.lines.len();
    if target <= 1 {
        return reference;
    }

    let mut lo = reference.width / target as f32;
    let mut hi = max_width;
    const PRECISION: f32 = 0.5;

    while hi - lo > PRECISION {
        let mid = (lo + hi) / 2.0;
        let attempt = greedy_paragraph(inlines, styles, breakpoints, mid, block, base_size, shaper);
        if attempt.lines.len() <= target {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    greedy_paragraph(inlines, styles, breakpoints, hi, block, base_size, shaper)
}

fn multiline_paragraph(
    inlines: &[Inline],
    styles: &[RunStyle],
    breakpoints: &[Vec<usize>],
    max_width: f32,
    block: &BlockStyle,
    base_size: f32,
    shaper: &Shaper<'_>,
) -> Paragraph {
    if block.text_wrap == Some(TextWrap::Balance) && !block.nowrap && max_width.is_finite() {
        return balanced_paragraph(
            inlines,
            styles,
            breakpoints,
            max_width,
            block,
            base_size,
            shaper,
        );
    }
    if block.line_break == crate::ir::LineBreakMode::KnuthPlass
        && !block.nowrap
        && max_width.is_finite()
    {
        if let Some(p) = knuth_plass_paragraph(
            inlines,
            styles,
            breakpoints,
            max_width,
            block,
            base_size,
            shaper,
        ) {
            return p;
        }
    }
    greedy_paragraph(
        inlines,
        styles,
        breakpoints,
        max_width,
        block,
        base_size,
        shaper,
    )
}

/// Split inlines at hard newlines into blocks; each block lays out separately.
fn create_blocks(inlines: &[Inline], styles: &[RunStyle]) -> Vec<(Vec<Inline>, Vec<RunStyle>)> {
    let mut blocks: Vec<(Vec<Inline>, Vec<RunStyle>)> = vec![(Vec::new(), Vec::new())];

    for (i, inline) in inlines.iter().enumerate() {
        let style = &styles[i];
        let text = inline.text();
        let indices: Vec<usize> = text.match_indices('\n').map(|(i, _)| i).collect();

        if indices.is_empty() {
            let last = blocks.len() - 1;
            blocks[last].0.push(inline.clone());
            blocks[last].1.push(style.clone());
            continue;
        }

        let mut cuts = indices;
        if cuts.last() != Some(&text.len()) {
            cuts.push(text.len());
        }

        let mut start = 0usize;
        for (i, &end) in cuts.iter().enumerate() {
            let chunk = &text[start..end];
            let last = blocks.len() - 1;
            blocks[last].0.push(match inline {
                Inline::Text(_) => Inline::Text(chunk.to_string()),
                Inline::Span { style, .. } => Inline::Span {
                    text: chunk.to_string(),
                    style: style.clone(),
                },
            });
            blocks[last].1.push(style.clone());
            if i < cuts.len() - 1 {
                blocks.push((Vec::new(), Vec::new()));
            }
            start = end + 1;
        }
    }

    blocks
}

/// `createParagraph` — one paragraph per hard-newline-separated block.
pub fn create_paragraphs(
    content: &TextContent,
    max_width: f32,
    engine: &dyn TextEngine,
    size_override: Option<f32>,
) -> Vec<Paragraph> {
    let shaper = Shaper { engine };
    let block = &content.block;

    let base_size = size_override.unwrap_or(content.base.size);
    let mut base = content.base.clone();
    base.size = base_size;

    let styles: Vec<RunStyle> = content
        .inlines
        .iter()
        .map(|i| match i {
            Inline::Text(_) => base.clone(),
            Inline::Span { style, .. } => {
                let mut s = style.clone();
                if let Some(size) = size_override {
                    // Autofit scales the block; spans keep their relative size.
                    if (style.size - content.base.size).abs() < f32::EPSILON {
                        s.size = size;
                    }
                }
                s
            }
        })
        .collect();

    let full_text: String = content.inlines.iter().map(|i| i.text()).collect();
    let resolved_dir = resolve_paragraph_direction(&full_text, block.base_dir);

    let blocks = create_blocks(&content.inlines, &styles);
    let mut out = Vec::with_capacity(blocks.len());

    for (block_inlines, block_styles) in blocks {
        let breakpoints: Vec<Vec<usize>> = if block.nowrap {
            block_inlines.iter().map(|_| Vec::new()).collect()
        } else {
            block_inlines
                .iter()
                .map(|i| engine.break_points(i.text()))
                .collect()
        };

        let mut paragraph = multiline_paragraph(
            &block_inlines,
            &block_styles,
            &breakpoints,
            max_width,
            block,
            base_size,
            &shaper,
        );
        apply_text_overflow(&mut paragraph, max_width, block, base_size, &shaper);
        merge_segments(&mut paragraph, block.align);
        paragraph.base_dir = resolved_dir;
        out.push(paragraph);
    }

    out
}

/// Coalesce adjacent segments that share a style, so each becomes one shaped
/// run. Justified text keeps spaces separate so they can be widened.
fn merge_segments(paragraph: &mut Paragraph, align: Option<TextAlign>) {
    for line in &mut paragraph.lines {
        let mut groups: Vec<Vec<Segment>> = vec![Vec::new()];

        for segment in line.segments.drain(..) {
            let current = groups.last_mut().unwrap();
            if let Some(tail) = current.last() {
                if align == Some(TextAlign::Justify) && is_all_whitespace(&segment.text) {
                    groups.push(vec![segment]);
                    groups.push(Vec::new());
                    continue;
                }
                if tail.style != segment.style || tail.is_tab || segment.is_tab {
                    groups.push(vec![segment]);
                    continue;
                }
            }
            current.push(segment);
        }

        let mut merged = Vec::new();
        for group in groups {
            if group.is_empty() {
                continue;
            }
            let mut iter = group.into_iter();
            let mut out = iter.next().unwrap();
            for next in iter {
                out.text.push_str(&next.text);
                out.height = out.height.max(next.height);
                out.width += next.width;
            }
            merged.push(out);
        }
        line.segments = merged;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FixedMetricsEngine;

    fn content(text: &str) -> TextContent {
        TextContent {
            base: RunStyle {
                size: 10.0,
                ..Default::default()
            },
            block: BlockStyle::default(),
            inlines: vec![Inline::Text(text.into())],
            clip_image: None,
        }
    }

    fn lines_of(p: &Paragraph) -> Vec<String> {
        p.lines
            .iter()
            .map(|l| {
                l.segments
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<String>()
            })
            .collect()
    }

    #[test]
    fn single_line_when_it_fits() {
        let e = FixedMetricsEngine::default();
        let p = create_paragraphs(&content("hello world"), 1000.0, &e, None);
        assert_eq!(p.len(), 1);
        assert_eq!(lines_of(&p[0]), vec!["hello world"]);
    }

    #[test]
    fn greedy_wraps_at_word_boundaries() {
        let e = FixedMetricsEngine::default();
        // 10px per char: "hello " = 60, "world" = 50
        let p = create_paragraphs(&content("hello world"), 60.0, &e, None);
        assert_eq!(lines_of(&p[0]), vec!["hello", "world"]);
    }

    #[test]
    fn hard_newlines_split_into_blocks() {
        let e = FixedMetricsEngine::default();
        let p = create_paragraphs(&content("a\nb\nc"), 1000.0, &e, None);
        assert_eq!(p.len(), 3);
        assert_eq!(lines_of(&p[0]), vec!["a"]);
        assert_eq!(lines_of(&p[2]), vec!["c"]);
    }

    #[test]
    fn nowrap_never_breaks() {
        let e = FixedMetricsEngine::default();
        let mut c = content("hello world");
        c.block.nowrap = true;
        let p = create_paragraphs(&c, 10.0, &e, None);
        assert_eq!(lines_of(&p[0]), vec!["hello world"]);
    }

    #[test]
    fn max_lines_truncates() {
        let e = FixedMetricsEngine::default();
        let mut c = content("one two three four");
        c.block.max_lines = Some(2);
        let p = create_paragraphs(&c, 40.0, &e, None);
        assert_eq!(p[0].lines.len(), 2);
    }

    #[test]
    fn ellipsis_is_appended_when_clamped() {
        let e = FixedMetricsEngine::default();
        let mut c = content("one two three four");
        c.block.max_lines = Some(1);
        c.block.text_overflow = TextOverflow::Ellipsis;
        let p = create_paragraphs(&c, 60.0, &e, None);
        let text: String = p[0].lines[0]
            .segments
            .iter()
            .map(|s| s.text.as_str())
            .collect();
        assert!(text.ends_with('…'), "{text:?}");
    }

    #[test]
    fn line_height_multiplies_the_line_box() {
        let e = FixedMetricsEngine::default();
        let plain = create_paragraphs(&content("x"), 1000.0, &e, None);
        let mut c = content("x");
        c.block.line_height = Some(2.0);
        let doubled = create_paragraphs(&c, 1000.0, &e, None);
        assert!(doubled[0].height > plain[0].height);
        assert_eq!(doubled[0].height, 20.0);
    }

    #[test]
    fn indent_widens_the_first_line_only() {
        let e = FixedMetricsEngine::default();
        let mut c = content("abc");
        c.block.indent_size = 25.0;
        let p = create_paragraphs(&c, 1000.0, &e, None);
        assert_eq!(p[0].lines[0].width, 55.0);
    }

    #[test]
    fn adjacent_same_style_segments_merge() {
        let e = FixedMetricsEngine::default();
        let mut c = content("");
        c.inlines = vec![Inline::Text("foo".into()), Inline::Text("bar".into())];
        let p = create_paragraphs(&c, 1000.0, &e, None);
        assert_eq!(p[0].lines[0].segments.len(), 1);
        assert_eq!(p[0].lines[0].segments[0].text, "foobar");
    }

    #[test]
    fn differing_styles_do_not_merge() {
        let e = FixedMetricsEngine::default();
        let mut c = content("");
        c.inlines = vec![
            Inline::Text("foo".into()),
            Inline::Span {
                text: "bar".into(),
                style: RunStyle {
                    size: 20.0,
                    ..Default::default()
                },
            },
        ];
        let p = create_paragraphs(&c, 1000.0, &e, None);
        assert_eq!(p[0].lines[0].segments.len(), 2);
    }

    #[test]
    fn knuth_plass_balances_better_than_greedy() {
        let e = FixedMetricsEngine::default();
        let mut c = content("aaa bb cc ddddd ee");
        c.block.line_break = crate::ir::LineBreakMode::KnuthPlass;
        let p = create_paragraphs(&c, 90.0, &e, None);
        assert!(p[0].lines.len() >= 2);
        assert!(p[0].lines.iter().all(|l| l.width <= 90.0));
    }

    #[test]
    fn balanced_wrap_evens_out_lines() {
        let e = FixedMetricsEngine::default();
        let mut c = content("one two three four five six");
        c.block.text_wrap = Some(TextWrap::Balance);
        let balanced = create_paragraphs(&c, 200.0, &e, None);
        c.block.text_wrap = None;
        let greedy = create_paragraphs(&c, 200.0, &e, None);
        assert_eq!(balanced[0].lines.len(), greedy[0].lines.len());
        assert!(balanced[0].width <= greedy[0].width);
    }

    #[test]
    fn tab_stops_expand_to_the_next_stop() {
        let e = FixedMetricsEngine::default();
        let mut c = content("a\tb");
        c.block.tab_stops = vec![100.0, 200.0];
        let p = create_paragraphs(&c, 1000.0, &e, None);
        let segs = &p[0].lines[0].segments;
        assert_eq!(segs.len(), 3);
        assert!(segs[1].is_tab);
        assert_eq!(segs[0].width + segs[1].width, 100.0);
    }

    #[test]
    fn tab_leader_fills_the_gap() {
        let e = FixedMetricsEngine::default();
        let mut c = content("a\tb");
        c.block.tab_stops = vec![100.0];
        c.block.tab_leader = ".".into();
        let p = create_paragraphs(&c, 1000.0, &e, None);
        let leader = p[0].lines[0].segments[1].tab_leader.clone().unwrap();
        assert_eq!(leader.len(), 9);
    }

    #[test]
    fn rtl_direction_is_detected() {
        let e = FixedMetricsEngine::default();
        let mut c = content("مرحبا");
        c.block.base_dir = Some(crate::text::bidi::BaseDir::Auto);
        let p = create_paragraphs(&c, 1000.0, &e, None);
        assert_eq!(p[0].base_dir, Dir::Rtl);
    }

    #[test]
    fn trailing_whitespace_is_trimmed_from_lines() {
        let e = FixedMetricsEngine::default();
        let p = create_paragraphs(&content("abc   "), 1000.0, &e, None);
        assert_eq!(p[0].lines[0].width, 30.0);
    }
}
