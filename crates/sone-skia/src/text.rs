use std::num::NonZeroUsize;
use std::sync::Mutex;

use lru::LruCache;
use skia_safe::font_arguments::variation_position::Coordinate;
use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextAlign, TextStyle,
};
use skia_safe::FontArguments;

use sone_core::paint::{SpanStyle, TextEngine, TextMetrics};
use sone_core::text::linebreak::{apply_break_rules, script_runs, uax29_word_starts};

use crate::fonts::{font_style, FontRegistry};

/// Layout width that cannot wrap; sone has already decided where runs end.
pub const UNCONSTRAINED: f32 = 1e7;

const MAX_WIDTH_ENTRIES: usize = 20_000;
const MAX_BREAK_ENTRIES: usize = 2_000;

pub struct SkiaTextEngine {
    pub fonts: FontRegistry,
    widths: Mutex<LruCache<String, f32>>,
    vertical: Mutex<LruCache<String, (f32, f32)>>,
    breaks: Mutex<LruCache<String, Vec<usize>>>,
}

impl Default for SkiaTextEngine {
    fn default() -> Self {
        SkiaTextEngine::new()
    }
}

impl SkiaTextEngine {
    pub fn new() -> Self {
        SkiaTextEngine {
            fonts: FontRegistry::new(),
            widths: Mutex::new(LruCache::new(NonZeroUsize::new(MAX_WIDTH_ENTRIES).unwrap())),
            vertical: Mutex::new(LruCache::new(NonZeroUsize::new(1024).unwrap())),
            breaks: Mutex::new(LruCache::new(NonZeroUsize::new(MAX_BREAK_ENTRIES).unwrap())),
        }
    }

    pub fn clear_caches(&self) {
        self.widths.lock().unwrap().clear();
        self.vertical.lock().unwrap().clear();
        self.breaks.lock().unwrap().clear();
    }

    pub fn text_style(&self, style: &SpanStyle) -> TextStyle {
        let mut ts = TextStyle::new();
        let families = self
            .fonts
            .resolve(&style.font)
            .unwrap_or_else(|_| style.font.clone());
        ts.set_font_families(&families);
        ts.set_font_size(style.size.max(0.0));
        ts.set_font_style(font_style(style.weight, style.italic, style.oblique));
        ts.set_letter_spacing(style.letter_spacing);
        ts.set_word_spacing(style.word_spacing);
        // Variable fonts need the axis set explicitly; `fontStyle.weight` alone
        // only selects among static faces.
        let coords = self.variation_coordinates(&families, style.weight);
        if !coords.is_empty() {
            let position = skia_safe::font_arguments::VariationPosition {
                coordinates: &coords,
            };
            let args = FontArguments::new();
            ts.set_font_arguments(&args.set_variation_design_position(position));
        }
        ts
    }

    /// Every axis the primary family exposes: `wght` from the requested weight,
    /// the rest pinned to their defaults. Leaving an axis out makes Skia clamp
    /// it to the axis minimum, which CanvasKit does not do.
    fn variation_coordinates(&self, families: &[String], weight: i32) -> Vec<Coordinate> {
        const WGHT: u32 = 0x77676874;
        let Some(primary) = families.first() else {
            return Vec::new();
        };
        let axes = self.fonts.axes(primary);
        let mut coords: Vec<Coordinate> = axes
            .iter()
            .map(|(tag, def)| Coordinate {
                axis: skia_safe::FourByteTag::from(*tag),
                value: if *tag == WGHT { weight as f32 } else { *def },
            })
            .collect();
        if !axes.iter().any(|(tag, _)| *tag == WGHT) {
            coords.push(Coordinate {
                axis: skia_safe::FourByteTag::from(WGHT),
                value: weight as f32,
            });
        }
        coords
    }

    fn collection(&self) -> FontCollection {
        self.fonts.collection()
    }

    fn shape_width(&self, text: &str, style: &SpanStyle) -> f32 {
        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&self.text_style(style));
        ps.set_text_align(TextAlign::Left);
        let mut builder = ParagraphBuilder::new(&ps, self.collection());
        builder.add_text(text);
        let mut paragraph = builder.build();
        paragraph.layout(UNCONSTRAINED);
        paragraph.max_intrinsic_width()
    }
}

impl TextEngine for SkiaTextEngine {
    fn measure(&self, text: &str, style: &SpanStyle) -> TextMetrics {
        let vkey = format!("{}|{}", style.font.join(","), style.size);
        let (ascent, descent) = {
            let mut cache = self.vertical.lock().unwrap();
            if let Some(v) = cache.get(&vkey) {
                *v
            } else {
                let v = match self.fonts.font_for(&style.font, style.size) {
                    Ok(font) => {
                        let (_, m) = font.metrics();
                        // Skia reports ascent as negative; CSS wants it positive.
                        (-m.ascent, m.descent)
                    }
                    Err(_) => (style.size * 0.8, style.size * 0.2),
                };
                cache.put(vkey, v);
                v
            }
        };

        if text.is_empty() {
            return TextMetrics {
                width: 0.0,
                ascent,
                descent,
            };
        }

        let key = format!("{}|{}", style.key(), text);
        let mut cache = self.widths.lock().unwrap();
        let width = if let Some(w) = cache.get(&key) {
            *w
        } else {
            let w = self.shape_width(text, style);
            cache.put(key, w);
            w
        };
        TextMetrics {
            width,
            ascent,
            descent,
        }
    }

    /// Latin-and-friends segmentation comes from UAX#29, which matches
    /// `Intl.Segmenter` exactly; only dictionary scripts (Khmer, Thai, Lao,
    /// Burmese) go through Skia's ICU, which is the only source of word
    /// boundaries for them.
    /// Latin and friends come from UAX#29, which matches `Intl.Segmenter`
    /// exactly. Khmer, Thai, Lao and Burmese have no UAX#29 word boundaries at
    /// all, so those runs go through Skia's bundled ICU instead.
    fn word_starts(&self, text: &str) -> Vec<usize> {
        if text.is_empty() {
            return Vec::new();
        }

        let mut starts = Vec::new();
        for (start, end, dictionary) in script_runs(text) {
            let run = &text[start..end];
            let run_starts = if dictionary {
                self.icu_word_starts(run)
            } else {
                uax29_word_starts(run)
            };
            for offset in run_starts {
                let at = start + offset;
                if starts.last() != Some(&at) {
                    starts.push(at);
                }
            }
        }
        if starts.first() != Some(&0) {
            starts.insert(0, 0);
        }
        starts
    }

    fn break_points(&self, text: &str) -> Vec<usize> {
        if let Some(hit) = self.breaks.lock().unwrap().get(text) {
            return hit.clone();
        }
        let points = apply_break_rules(text, &self.word_starts(text));
        self.breaks
            .lock()
            .unwrap()
            .put(text.to_string(), points.clone());
        points
    }

    fn has_font(&self, family: &str) -> bool {
        self.fonts.has(family)
    }
}

impl SkiaTextEngine {
    /// Word starts from Skia's ICU word iterator, for one dictionary-script run.
    fn icu_word_starts(&self, text: &str) -> Vec<usize> {
        let style = SpanStyle::default();
        let mut ps = ParagraphStyle::new();
        ps.set_text_style(&self.text_style(&style));
        ps.set_text_align(TextAlign::Left);
        let mut builder = ParagraphBuilder::new(&ps, self.collection());
        builder.add_text(text);
        let mut paragraph = builder.build();
        paragraph.layout(UNCONSTRAINED);

        // Skia reports word boundaries as UTF-16 offsets; sone works in bytes.
        let to_byte = utf16_offsets(text);
        let utf16_len = text.encode_utf16().count();

        let mut starts = Vec::new();
        let mut offset = 0usize;
        while offset < utf16_len {
            let range = paragraph.get_word_boundary(offset as u32);
            let start = range.start.min(utf16_len);
            let end = range.end.min(utf16_len);
            if let Some(&b) = to_byte.get(start) {
                if starts.last() != Some(&b) {
                    starts.push(b);
                }
            }
            if end <= offset {
                break;
            }
            offset = end;
        }
        starts.sort_unstable();
        starts.dedup();
        starts
    }
}

/// Byte offset that starts each UTF-16 code unit of `text`.
fn utf16_offsets(text: &str) -> Vec<usize> {
    let mut out = Vec::with_capacity(text.len() + 1);
    for (byte, ch) in text.char_indices() {
        for _ in 0..ch.len_utf16() {
            out.push(byte);
        }
    }
    out.push(text.len());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_mapping() {
        // 'a' is one unit and one byte; 'ក' is one unit and three bytes.
        assert_eq!(utf16_offsets("aក"), vec![0, 1, 4]);
        // An astral character takes two UTF-16 units.
        assert_eq!(utf16_offsets("a😀"), vec![0, 1, 1, 5]);
    }

    #[test]
    fn empty_text_measures_to_zero_width() {
        let e = SkiaTextEngine::new();
        let m = e.measure("", &SpanStyle::default());
        assert_eq!(m.width, 0.0);
    }
}
