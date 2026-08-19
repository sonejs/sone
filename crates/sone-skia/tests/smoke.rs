//! Capability probes for every skia-safe API the engine depends on, so a
//! version bump that drops one fails here rather than in a render.

use std::path::PathBuf;

use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider,
};
use skia_safe::{image_filters, Data, FontMgr, Paint, Rect};

fn font_bytes() -> Vec<u8> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/font/GeistMono-Regular.ttf");
    std::fs::read(path).unwrap()
}

fn collection() -> FontCollection {
    let mgr = FontMgr::new();
    let typeface = mgr
        .new_from_data(&font_bytes(), None)
        .expect("typeface from data");
    let mut provider = TypefaceFontProvider::new();
    provider.register_typeface(typeface, Some("GeistMono"));
    let mut collection = FontCollection::new();
    collection.set_asset_font_manager(Some(provider.into()));
    collection.enable_font_fallback();
    collection
}

#[test]
fn raster_surface_and_png_encoding() {
    let mut surface = skia_safe::surfaces::raster_n32_premul((8, 8)).unwrap();
    surface.canvas().clear(skia_safe::Color::RED);
    let image = surface.image_snapshot();
    let ctx: Option<&mut skia_safe::gpu::DirectContext> = None;
    assert!(image
        .encode(ctx, skia_safe::EncodedImageFormat::PNG, 100)
        .is_some());
}

#[test]
fn webp_and_jpeg_encoding() {
    let mut surface = skia_safe::surfaces::raster_n32_premul((8, 8)).unwrap();
    surface.canvas().clear(skia_safe::Color::BLUE);
    let image = surface.image_snapshot();
    for format in [
        skia_safe::EncodedImageFormat::WEBP,
        skia_safe::EncodedImageFormat::JPEG,
    ] {
        let ctx: Option<&mut skia_safe::gpu::DirectContext> = None;
        assert!(
            image.encode(ctx, format, 90).is_some(),
            "{format:?} encoder missing"
        );
    }
}

#[test]
fn paragraph_shaping_and_metrics() {
    let mut style = ParagraphStyle::new();
    let mut text_style = TextStyle::new();
    text_style.set_font_families(&["GeistMono"]);
    text_style.set_font_size(16.0);
    style.set_text_style(&text_style);

    let mut builder = ParagraphBuilder::new(&style, collection());
    builder.add_text("Hello world");
    let mut paragraph = builder.build();
    paragraph.layout(1e7);
    assert!(paragraph.max_intrinsic_width() > 0.0);
    assert!(!paragraph.get_line_metrics().is_empty());
}

#[test]
fn word_boundaries_are_available() {
    let mut style = ParagraphStyle::new();
    let mut text_style = TextStyle::new();
    text_style.set_font_families(&["GeistMono"]);
    text_style.set_font_size(16.0);
    style.set_text_style(&text_style);
    let mut builder = ParagraphBuilder::new(&style, collection());
    builder.add_text("hello world");
    let mut paragraph = builder.build();
    paragraph.layout(1e7);
    let range = paragraph.get_word_boundary(0);
    assert!(range.end > range.start);
}

#[test]
fn font_metrics_and_variation_axes() {
    let mgr = FontMgr::new();
    let typeface = mgr.new_from_data(&font_bytes(), None).unwrap();
    let font = skia_safe::Font::from_typeface(typeface, 16.0);
    let (_, metrics) = font.metrics();
    assert!(metrics.ascent < 0.0 && metrics.descent > 0.0);

    let variable = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/font/GoogleSans-VariableFont_GRAD,opsz,wght.ttf");
    let typeface = mgr
        .new_from_data(&std::fs::read(variable).unwrap(), None)
        .unwrap();
    let axes = typeface
        .variation_design_parameters()
        .expect("variation axes");
    assert!(
        axes.iter().any(|a| *a.tag == 0x77676874),
        "wght axis missing"
    );
}

#[test]
fn pdf_document_round_trips() {
    let mut buffer: Vec<u8> = Vec::new();
    {
        let mut document = skia_safe::pdf::new_document(&mut buffer, None);
        let mut page = document.begin_page((100.0, 100.0), None);
        page.canvas()
            .draw_circle((50.0, 50.0), 20.0, &Paint::default());
        document = page.end_page();
        document.close();
    }
    assert!(buffer.starts_with(b"%PDF"));
}

#[test]
fn svg_canvas_round_trips() {
    let canvas = skia_safe::svg::Canvas::new(Rect::from_wh(20.0, 20.0), None);
    canvas.draw_circle((10.0, 10.0), 5.0, &Paint::default());
    let data = canvas.end();
    let text = String::from_utf8_lossy(data.as_bytes()).to_string();
    assert!(text.contains("<svg"), "{text}");
}

#[test]
fn svg_dom_rasterizes() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#;
    let dom = skia_safe::svg::Dom::from_bytes(&Data::new_copy(svg), FontMgr::new()).unwrap();
    assert!(dom.root().intrinsic_size().width > 0.0);
}

#[test]
fn drop_shadow_only_filter_exists() {
    assert!(image_filters::drop_shadow_only(
        (1.0, 1.0),
        (2.0, 2.0),
        skia_safe::Color::BLACK,
        None,
        None,
        None
    )
    .is_some());
}

#[test]
fn gradient_shaders_build() {
    use skia_safe::gradient::{shaders, Colors, Gradient, Interpolation};
    let colors = [
        skia_safe::Color4f::new(1.0, 0.0, 0.0, 1.0),
        skia_safe::Color4f::new(0.0, 0.0, 1.0, 1.0),
    ];
    let stops = [0.0f32, 1.0];
    let gc = Colors::new(&colors, Some(&stops), skia_safe::TileMode::Clamp, None);
    let gradient = Gradient::new(gc, Interpolation::default());
    assert!(shaders::linear_gradient(((0.0, 0.0), (10.0, 10.0)), &gradient, None).is_some());
    assert!(shaders::radial_gradient(((5.0, 5.0), 5.0), &gradient, None).is_some());
}

#[test]
fn dash_path_effect_exists() {
    assert!(skia_safe::PathEffect::dash(&[4.0, 4.0], 0.0).is_some());
}
