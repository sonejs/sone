use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sone_core::compile::AssetLoader;
use sone_core::error::SoneError;
use sone_core::paint::ImageHandle;
use sone_core::Result;

/// Resolves `file:`/relative, `data:` and registered `asset:` sources.
pub struct Assets {
    base_dir: PathBuf,
    cache: Mutex<HashMap<String, ImageHandle>>,
    registered: Mutex<HashMap<String, Vec<u8>>>,
}

impl Assets {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Assets {
            base_dir: base_dir.into(),
            cache: Mutex::new(HashMap::new()),
            registered: Mutex::new(HashMap::new()),
        }
    }

    /// Register bytes under `asset:<name>`, for FFI callers with no filesystem.
    pub fn register(&self, name: &str, bytes: Vec<u8>) {
        self.registered
            .lock()
            .unwrap()
            .insert(name.to_string(), bytes);
    }

    pub fn read(&self, src: &str) -> Result<Vec<u8>> {
        if let Some(rest) = src.strip_prefix("data:") {
            let payload =
                rest.split_once("base64,")
                    .map(|(_, b)| b)
                    .ok_or_else(|| SoneError::Asset {
                        src: src.to_string(),
                        message: "only base64 data URLs are supported".into(),
                    })?;
            return decode_base64(payload).ok_or_else(|| SoneError::Asset {
                src: src.to_string(),
                message: "invalid base64 payload".into(),
            });
        }
        if let Some(name) = src.strip_prefix("asset:") {
            return self
                .registered
                .lock()
                .unwrap()
                .get(name)
                .cloned()
                .ok_or_else(|| SoneError::Asset {
                    src: src.to_string(),
                    message: "no such registered asset".into(),
                });
        }
        if src.starts_with("http://") || src.starts_with("https://") {
            return Err(SoneError::Asset {
                src: src.to_string(),
                message:
                    "remote assets must be fetched by the caller and registered as asset:<name>"
                        .into(),
            });
        }
        let path = self.resolve_path(src);
        std::fs::read(&path).map_err(|e| SoneError::Asset {
            src: path.display().to_string(),
            message: e.to_string(),
        })
    }

    pub fn resolve_path(&self, src: &str) -> PathBuf {
        let raw = src
            .strip_prefix("file://")
            .or_else(|| src.strip_prefix("file:"))
            .unwrap_or(src);
        let p = Path::new(raw);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.base_dir.join(p)
        }
    }
}

impl AssetLoader for Assets {
    fn load_image(&self, src: &str) -> Result<ImageHandle> {
        if let Some(hit) = self.cache.lock().unwrap().get(src) {
            return Ok(hit.clone());
        }
        let bytes = self.read(src)?;
        let handle = crate::image::decode(&bytes, src)?;
        self.cache
            .lock()
            .unwrap()
            .insert(src.to_string(), handle.clone());
        Ok(handle)
    }
}

fn decode_base64(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, c) in TABLE.iter().enumerate() {
        lookup[*c as usize] = i as u8;
    }

    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for b in input.bytes() {
        if b == b'=' || b.is_ascii_whitespace() {
            continue;
        }
        let v = lookup[b as usize];
        if v == 255 {
            return None;
        }
        buffer = (buffer << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_known_values() {
        assert_eq!(decode_base64("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(decode_base64("").unwrap(), b"");
        assert!(decode_base64("!!!").is_none());
    }

    #[test]
    fn relative_paths_resolve_against_the_base() {
        let a = Assets::new("/tmp/base");
        assert_eq!(
            a.resolve_path("img/x.png"),
            PathBuf::from("/tmp/base/img/x.png")
        );
        assert_eq!(
            a.resolve_path("file:./y.png"),
            PathBuf::from("/tmp/base/./y.png")
        );
        assert_eq!(a.resolve_path("/abs/z.png"), PathBuf::from("/abs/z.png"));
    }
}
