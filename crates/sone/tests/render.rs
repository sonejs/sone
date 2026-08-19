//! Rendering, and the parity gate every sone binding owes.

use std::path::{Path, PathBuf};
use std::process::Command;

use sone::prelude::*;
use sone::{Engine, Granularity};

fn root_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir
}

const FAMILY: &str = "Geist Mono";

fn font_path() -> PathBuf {
    root_dir().join("fixtures/font/GeistMono-Regular.ttf")
}

fn engine() -> Engine {
    let engine = Engine::new(root_dir());
    engine.register_font_file(FAMILY, font_path()).unwrap();
    engine
}

fn card() -> Column {
    column()
        .gap(20)
        .padding(20)
        .size(420, 200)
        .bg("khaki")
        .corner_radius(28)
        .child(
            text("Hello ")
                .font_family(FAMILY)
                .size(24)
                .line_height(1.4)
                .span(span("world").weight("bold").color("#c0392b")),
        )
        .child(
            row()
                .gap(10)
                .child(column().bg("lightgreen").square(50).corner_radius(14))
                .child(column().bg("salmon").height(50).corner_radius(14).flex(1)),
        )
}

#[test]
fn renders_a_png() {
    let engine = engine();
    let png = sone::render(column().square(16).bg("red"))
        .engine(&engine)
        .png()
        .unwrap();
    assert_eq!(&png[..4], b"\x89PNG");
}

#[test]
fn density_scales_the_raster() {
    let engine = engine();
    // Raw is 4 bytes per pixel, so the byte count is the pixel count.
    let one = sone::render(column().square(10).bg("red"))
        .engine(&engine)
        .raw()
        .unwrap();
    let two = sone::render(column().square(10).bg("red"))
        .engine(&engine)
        .density(2)
        .raw()
        .unwrap();
    assert_eq!(one.len(), 10 * 10 * 4);
    assert_eq!(two.len(), 20 * 20 * 4);
}

#[test]
fn renders_every_format() {
    let engine = engine();
    let pdf = sone::render(card()).engine(&engine).pdf().unwrap();
    assert_eq!(&pdf[..4], b"%PDF");

    let svg = sone::render(card()).engine(&engine).svg().unwrap();
    assert!(String::from_utf8_lossy(&svg).contains("<svg"));

    assert!(!sone::render(card()).engine(&engine).jpeg().unwrap().is_empty());
    assert!(!sone::render(card()).engine(&engine).webp().unwrap().is_empty());
}

#[test]
fn one_page_per_declared_break() {
    let engine = engine();
    let root = column()
        .child(column().height(60).bg("red"))
        .child(column().height(60).bg("green").page_break(PageBreak::Before))
        .child(column().height(60).bg("blue").page_break(PageBreak::Before));

    let pages = sone::render(root)
        .engine(&engine)
        .width(40)
        .page_height(200)
        .pages()
        .unwrap();
    assert_eq!(pages.len(), 3);
}

#[test]
fn the_font_registry_round_trips() {
    let engine = Engine::new(root_dir());
    assert!(!engine.has_font(FAMILY));
    engine.register_font_file(FAMILY, font_path()).unwrap();
    assert!(engine.has_font(FAMILY));
    assert!(engine.font_families().iter().any(|name| name == FAMILY));
    engine.reset_fonts();
    assert!(!engine.has_font(FAMILY));
}

#[test]
fn registered_images_resolve_as_assets() {
    let engine = engine();
    let png = sone::render(column().square(8).bg("red"))
        .engine(&engine)
        .png()
        .unwrap();
    engine.register_image("logo", png);
    assert!(!sone::render(photo("asset:logo").square(8))
        .engine(&engine)
        .png()
        .unwrap()
        .is_empty());
}

#[test]
fn layout_comes_back_as_a_tree() {
    let engine = engine();
    let layout = sone::render(column().padding(5).child(column().square(20).tag("inner")))
        .engine(&engine)
        .layout()
        .unwrap();
    assert_eq!(layout["width"], serde_json::json!(30.0));
    assert_eq!(layout["children"][0]["tag"], serde_json::json!("inner"));
}

#[test]
fn metadata_honours_granularity() {
    let engine = engine();
    let rendering = sone::render(text("hello world").font_family(FAMILY).size(12));
    let rendering = rendering.engine(&engine);
    assert!(rendering.metadata(Granularity::Node).unwrap().is_object());
    assert!(rendering.metadata(Granularity::Word).unwrap().is_object());
}

#[test]
fn a_missing_asset_is_an_error() {
    let engine = engine();
    let result = sone::render(photo("does-not-exist.png").square(10))
        .engine(&engine)
        .png();
    assert!(result.is_err(), "a missing image should not render");
}

#[test]
fn an_unregistered_font_falls_back_rather_than_failing() {
    // Worth pinning down: the engine substitutes rather than erroring, so a
    // typo in a family name is a visual bug, not a caught one.
    let engine = engine();
    assert!(sone::render(text("hello").font_family("Nothing Here").size(12))
        .engine(&engine)
        .png()
        .is_ok());
}

#[test]
fn save_infers_the_format_from_the_extension() {
    let engine = engine();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("card.pdf");
    sone::render(card()).engine(&engine).save(&path).unwrap();
    assert_eq!(&std::fs::read(&path).unwrap()[..4], b"%PDF");

    let bad = dir.path().join("card.tiff");
    assert!(sone::render(card()).engine(&engine).save(&bad).is_err());
}

/// The gate every binding owes: the same document must come out of this crate
/// byte for byte the way it comes out of `sone-cli`.
#[test]
fn matches_the_cli_byte_for_byte() {
    let engine = engine();
    // An absolute src, because the CLI resolves a document's assets against the
    // document's own directory and the engine resolves them against its base
    // directory — the two only agree when the path is absolute.
    let rendering = sone::render(card())
        .engine(&engine)
        .density(2)
        .font(FAMILY, font_path().to_string_lossy());

    let dir = tempfile::tempdir().unwrap();
    let document = dir.path().join("doc.json");
    let from_cli = dir.path().join("cli.png");
    std::fs::write(&document, rendering.to_json_pretty()).unwrap();

    let status = Command::new(env!("CARGO"))
        .args(["run", "-q", "-p", "sone-cli", "--", "render"])
        .arg(&document)
        .args(["--density", "2", "-o"])
        .arg(&from_cli)
        .current_dir(root_dir())
        .status()
        .expect("sone-cli");
    assert!(status.success());

    assert_eq!(std::fs::read(&from_cli).unwrap(), rendering.png().unwrap());
    assert!(Path::new(&from_cli).exists());
}
