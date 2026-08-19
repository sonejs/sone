//! Line-break agreement with the TypeScript engine's `Intl.Segmenter`.
//!
//! Scripts segmented by UAX#29 must agree exactly. Dictionary scripts (Khmer,
//! Thai, Lao, Burmese) depend on the ICU dictionary that ships with the
//! runtime, and V8's differs from ICU4X's, so those are tracked as a ratio
//! rather than required to match — see `rust/goldens-waivers.toml`.

use std::path::PathBuf;

use serde::Deserialize;
use sone_core::paint::TextEngine;
use sone_skia::SkiaTextEngine;

#[derive(Deserialize)]
struct Entry {
    text: String,
    /// UTF-16 offsets, as produced by `Intl.Segmenter`.
    breaks: Vec<usize>,
}

/// Ratchet: dictionary-script agreement must not regress below this.
const DICTIONARY_FLOOR: f64 = 0.33;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn is_dictionary_script(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c as u32,
            0x1780..=0x17FF | 0x19E0..=0x19FF | 0x0E00..=0x0E7F | 0x0E80..=0x0EFF | 0x1000..=0x109F)
    })
}

/// Byte offset for each UTF-16 offset in `text`.
fn utf16_to_byte(text: &str, offsets: &[usize]) -> Vec<usize> {
    let mut map = Vec::with_capacity(text.len() + 1);
    for (byte, ch) in text.char_indices() {
        for _ in 0..ch.len_utf16() {
            map.push(byte);
        }
    }
    map.push(text.len());
    offsets
        .iter()
        .filter_map(|o| map.get(*o).copied())
        .collect()
}

fn corpus() -> Option<Vec<Entry>> {
    let path = root().join("fixtures/visual/break-corpus.json");
    let json = std::fs::read_to_string(&path).ok()?;
    Some(serde_json::from_str(&json).unwrap())
}

#[test]
fn uax29_scripts_match_exactly() {
    let Some(corpus) = corpus() else {
        eprintln!("no corpus; run `tools/sync-fixtures.sh <path-to-sone-checkout>`");
        return;
    };
    let engine = SkiaTextEngine::new();

    let mut mismatches = Vec::new();
    for entry in &corpus {
        if is_dictionary_script(&entry.text) {
            continue;
        }
        let expected = utf16_to_byte(&entry.text, &entry.breaks);
        let actual = engine.break_points(&entry.text);
        if expected != actual {
            mismatches.push(format!(
                "{:?}\n  ts   {expected:?}\n  rust {actual:?}",
                entry.text
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} of {} non-dictionary strings differ:\n{}",
        mismatches.len(),
        corpus.len(),
        mismatches
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn dictionary_scripts_hold_their_ratchet() {
    let Some(corpus) = corpus() else { return };
    let engine = SkiaTextEngine::new();

    let mut matched = 0;
    let mut total = 0;
    for entry in &corpus {
        if !is_dictionary_script(&entry.text) {
            continue;
        }
        total += 1;
        if utf16_to_byte(&entry.text, &entry.breaks) == engine.break_points(&entry.text) {
            matched += 1;
        }
    }
    let ratio = matched as f64 / total.max(1) as f64;
    eprintln!(
        "dictionary-script agreement: {matched}/{total} ({:.1}%)",
        ratio * 100.0
    );
    assert!(
        ratio >= DICTIONARY_FLOOR,
        "dictionary agreement {:.1}% fell below the {:.0}% ratchet",
        ratio * 100.0,
        DICTIONARY_FLOOR * 100.0
    );
}
