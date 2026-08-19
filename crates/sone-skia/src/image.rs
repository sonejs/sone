use std::sync::Arc;

use skia_safe::{Data, Image};

use sone_core::error::SoneError;
use sone_core::paint::ImageHandle;
use sone_core::Result;

/// Upper bound on a rasterized SVG side, so a malicious `width`/`height`
/// cannot ask for a multi-gigabyte surface.
pub const MAX_SVG_SIDE: f32 = 8192.0;

pub struct SkiaImage(pub Image);

/// Decode raster bytes, or rasterize an SVG document.
pub fn decode(bytes: &[u8], src: &str) -> Result<ImageHandle> {
    if looks_like_svg(bytes) {
        return rasterize_svg(bytes, src);
    }
    let data = Data::new_copy(bytes);
    let image = Image::from_encoded(data).ok_or_else(|| SoneError::Asset {
        src: src.to_string(),
        message: "unsupported or corrupt image data".into(),
    })?;
    Ok(ImageHandle {
        width: image.width() as u32,
        height: image.height() as u32,
        inner: Arc::new(SkiaImage(image)),
    })
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(512)];
    let text = String::from_utf8_lossy(head);
    let trimmed = text.trim_start();
    trimmed.starts_with("<svg")
        || trimmed.starts_with("<?xml")
        || trimmed.starts_with("<!DOCTYPE svg")
}

fn rasterize_svg(bytes: &[u8], src: &str) -> Result<ImageHandle> {
    use skia_safe::svg::Dom;

    let data = Data::new_copy(bytes);
    let font_mgr = skia_safe::FontMgr::new();
    let mut dom = Dom::from_bytes(&data, font_mgr).map_err(|_| SoneError::Asset {
        src: src.to_string(),
        message: "could not parse the SVG document".into(),
    })?;

    let intrinsic = dom.root().intrinsic_size();
    let mut width = intrinsic.width;
    let mut height = intrinsic.height;
    if width <= 0.0 || height <= 0.0 || width.is_nan() || height.is_nan() {
        width = 300.0;
        height = 150.0;
    }
    // Clamp while preserving the aspect ratio.
    let scale = (MAX_SVG_SIDE / width.max(height)).min(1.0);
    let w = (width * scale).round().max(1.0);
    let h = (height * scale).round().max(1.0);

    let mut surface =
        skia_safe::surfaces::raster_n32_premul((w as i32, h as i32)).ok_or_else(|| {
            SoneError::Asset {
                src: src.to_string(),
                message: "could not allocate an SVG surface".into(),
            }
        })?;
    dom.set_container_size((w, h));
    surface.canvas().scale((scale, scale));
    dom.render(surface.canvas());
    let image = surface.image_snapshot();

    Ok(ImageHandle {
        width: image.width() as u32,
        height: image.height() as u32,
        inner: Arc::new(SkiaImage(image)),
    })
}

pub fn as_skia(handle: &ImageHandle) -> Option<&Image> {
    handle.inner.downcast_ref::<SkiaImage>().map(|i| &i.0)
}
