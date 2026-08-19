use std::collections::HashMap;
use std::sync::Mutex;

use skia_safe::textlayout::{FontCollection, TypefaceFontProvider};
use skia_safe::{Font, FontMgr, FontStyle, Typeface};

use sone_core::error::SoneError;
use sone_core::Result;

/// CSS generic families sone treats as "whatever is registered".
const GENERIC: &[&str] = &[
    "sans-serif",
    "serif",
    "monospace",
    "cursive",
    "fantasy",
    "system-ui",
];

struct Inner {
    /// Insertion-ordered: the first registered family backs generic names.
    order: Vec<String>,
    faces: HashMap<String, Vec<Vec<u8>>>,
    aliases: HashMap<String, String>,
    provider: Option<TypefaceFontProvider>,
    collection: Option<FontCollection>,
    fonts: HashMap<String, Font>,
    /// Variation axes per family, so unspecified ones can be pinned to their
    /// defaults — CanvasKit does this implicitly, bare Skia does not.
    axes: HashMap<String, Vec<(u32, f32)>>,
}

pub struct FontRegistry {
    inner: Mutex<Inner>,
}

// Skia's refcounted handles are internally thread-safe; the Mutex serializes
// every mutation of the registry itself.
unsafe impl Send for FontRegistry {}
unsafe impl Sync for FontRegistry {}

impl Default for FontRegistry {
    fn default() -> Self {
        FontRegistry::new()
    }
}

impl FontRegistry {
    pub fn new() -> Self {
        FontRegistry {
            inner: Mutex::new(Inner {
                order: Vec::new(),
                faces: HashMap::new(),
                aliases: HashMap::new(),
                provider: None,
                collection: None,
                fonts: HashMap::new(),
                axes: HashMap::new(),
            }),
        }
    }

    pub fn register(&self, family: &str, bytes: Vec<u8>) -> Result<()> {
        let mgr = FontMgr::new();
        if mgr.new_from_data(&bytes, None).is_none() {
            return Err(SoneError::Font {
                family: family.to_string(),
                message: "could not parse the font data — is it a valid TTF/OTF?".into(),
            });
        }
        let mut inner = self.inner.lock().unwrap();
        if !inner.faces.contains_key(family) {
            inner.order.push(family.to_string());
        }
        inner
            .faces
            .entry(family.to_string())
            .or_default()
            .push(bytes);
        invalidate(&mut inner);
        Ok(())
    }

    pub fn unregister(&self, family: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner.faces.remove(family);
        inner.order.retain(|f| f != family);
        invalidate(&mut inner);
    }

    pub fn reset(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.faces.clear();
        inner.order.clear();
        inner.aliases.clear();
        invalidate(&mut inner);
    }

    /// Point a generic family name at a specific registered family.
    pub fn alias(&self, generic: &str, family: &str) {
        let mut inner = self.inner.lock().unwrap();
        inner
            .aliases
            .insert(generic.to_string(), family.to_string());
        invalidate(&mut inner);
    }

    pub fn has(&self, family: &str) -> bool {
        self.inner.lock().unwrap().faces.contains_key(family)
    }

    pub fn families(&self) -> Vec<String> {
        self.inner.lock().unwrap().order.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().order.is_empty()
    }

    /// Resolve a requested stack to registered families, then append every
    /// other family as fallback so a Latin primary still renders Khmer.
    pub fn resolve(&self, requested: &[String]) -> Result<Vec<String>> {
        let inner = self.inner.lock().unwrap();
        if inner.order.is_empty() {
            return Err(SoneError::Font {
                family: requested.first().cloned().unwrap_or_default(),
                message: "no fonts registered — every typeface must be loaded explicitly".into(),
            });
        }

        let mut resolved: Vec<String> = Vec::new();
        let add = |resolved: &mut Vec<String>, family: &str| {
            if !resolved.iter().any(|f| f == family) {
                resolved.push(family.to_string());
            }
        };

        for family in requested {
            if inner.faces.contains_key(family) {
                add(&mut resolved, family);
            } else if GENERIC.contains(&family.as_str()) {
                let aliased = inner.aliases.get(family);
                match aliased {
                    Some(a) if inner.faces.contains_key(a) => add(&mut resolved, a),
                    _ => add(&mut resolved, &inner.order[0]),
                }
            }
        }
        for family in &inner.order {
            add(&mut resolved, family);
        }
        Ok(resolved)
    }

    pub fn collection(&self) -> FontCollection {
        let mut inner = self.inner.lock().unwrap();
        rebuild(&mut inner);
        inner.collection.clone().unwrap()
    }

    /// A `Font` for the primary family, used only for `metrics()`.
    pub fn font_for(&self, families: &[String], size: f32) -> Result<Font> {
        let resolved = self.resolve(families)?;
        let primary = resolved[0].clone();
        let key = format!("{primary}|{size}");

        let mut inner = self.inner.lock().unwrap();
        if let Some(font) = inner.fonts.get(&key) {
            return Ok(font.clone());
        }
        let bytes = inner
            .faces
            .get(&primary)
            .and_then(|v| v.first())
            .cloned()
            .ok_or_else(|| SoneError::Font {
                family: primary.clone(),
                message: "not registered".into(),
            })?;
        let typeface: Typeface =
            FontMgr::new()
                .new_from_data(&bytes, None)
                .ok_or_else(|| SoneError::Font {
                    family: primary.clone(),
                    message: "could not parse the font data".into(),
                })?;
        let font = Font::from_typeface(typeface, size);
        inner.fonts.insert(key, font.clone());
        Ok(font)
    }

    /// `(tag, default)` for every variation axis the family exposes.
    pub fn axes(&self, family: &str) -> Vec<(u32, f32)> {
        {
            let inner = self.inner.lock().unwrap();
            if let Some(hit) = inner.axes.get(family) {
                return hit.clone();
            }
        }
        let axes = self
            .typeface(family)
            .and_then(|tf| tf.variation_design_parameters())
            .map(|params| params.iter().map(|a| (*a.tag, a.def)).collect::<Vec<_>>())
            .unwrap_or_default();
        self.inner
            .lock()
            .unwrap()
            .axes
            .insert(family.to_string(), axes.clone());
        axes
    }

    pub fn typeface(&self, family: &str) -> Option<Typeface> {
        let inner = self.inner.lock().unwrap();
        let bytes = inner.faces.get(family)?.first()?.clone();
        FontMgr::new().new_from_data(&bytes, None)
    }
}

fn invalidate(inner: &mut Inner) {
    inner.provider = None;
    inner.collection = None;
    inner.fonts.clear();
    inner.axes.clear();
}

fn rebuild(inner: &mut Inner) {
    if inner.collection.is_some() {
        return;
    }
    let mgr = FontMgr::new();
    let mut provider = TypefaceFontProvider::new();
    for family in &inner.order {
        for bytes in inner.faces.get(family).into_iter().flatten() {
            if let Some(tf) = mgr.new_from_data(bytes, None) {
                provider.register_typeface(tf, Some(family.as_str()));
            }
        }
    }
    let mut collection = FontCollection::new();
    collection.set_asset_font_manager(Some(provider.clone().into()));
    collection.enable_font_fallback();
    inner.provider = Some(provider);
    inner.collection = Some(collection);
}

/// CSS weight + slant mapped to a Skia `FontStyle`.
pub fn font_style(weight: i32, italic: bool, oblique: bool) -> FontStyle {
    use skia_safe::font_style::{Slant, Weight, Width};
    let slant = if italic {
        Slant::Italic
    } else if oblique {
        Slant::Oblique
    } else {
        Slant::Upright
    };
    FontStyle::new(Weight::from(weight), Width::NORMAL, slant)
}
