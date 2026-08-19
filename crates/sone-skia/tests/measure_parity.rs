use std::path::PathBuf;

use sone_core::paint::{SpanStyle, TextEngine};
use sone_skia::SkiaTextEngine;

fn fonts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/font")
}

fn engine() -> SkiaTextEngine {
    let e = SkiaTextEngine::new();
    for (name, file) in [
        ("GoogleSans", "GoogleSans-VariableFont_GRAD,opsz,wght.ttf"),
        ("Geist", "GeistMono-Regular.ttf"),
        ("GeistMono", "GeistMono-Regular.ttf"),
        ("GeistMono-Bold", "GeistMono-Bold.ttf"),
        ("Moul", "Moul-Regular.ttf"),
        ("NotoSansKhmer", "NotoSansKhmer.ttf"),
        ("NotoSansArabic", "NotoSansArabic.ttf"),
        ("NotoSansHebrew", "NotoSansHebrew.ttf"),
    ] {
        let bytes = std::fs::read(fonts_dir().join(file)).unwrap();
        e.fonts.register(name, bytes).unwrap();
    }
    e
}

/// Widths and vertical metrics dumped from the TS engine (CanvasKit).
const CASES: &[(&str, &str, f32, i32, f32, f32, f32)] = &[
    (
        "All systems stable. Dispatch pressure is down 18% from yesterday.",
        "GoogleSans",
        34.0,
        700,
        1_072.73,
        32.844,
        9.72400,
    ),
    (
        "Hello world",
        "GoogleSans",
        16.0,
        400,
        80.78,
        15.456,
        4.57600,
    ),
    ("Hello world", "GeistMono", 16.0, 400, 105.6, 16.08, 4.72000),
    (
        "ភាសាខ្មែរ",
        "NotoSansKhmer",
        20.0,
        400,
        67.00000,
        21.38,
        5.86000,
    ),
];

#[test]
fn widths_match_the_typescript_engine() {
    let e = engine();
    let mut failures = Vec::new();
    for (text, family, size, weight, width, ascent, descent) in CASES {
        let style = SpanStyle {
            font: vec![(*family).into()],
            size: *size,
            weight: *weight,
            ..Default::default()
        };
        let m = e.measure(text, &style);
        if (m.width - width).abs() > 0.05 {
            failures.push(format!(
                "width {text:?} {family}@{size}: got {} want {width}",
                m.width
            ));
        }
        if (m.ascent - ascent).abs() > 0.05 {
            failures.push(format!(
                "ascent {text:?} {family}@{size}: got {} want {ascent}",
                m.ascent
            ));
        }
        if (m.descent - descent).abs() > 0.05 {
            failures.push(format!(
                "descent {text:?} {family}@{size}: got {} want {descent}",
                m.descent
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
