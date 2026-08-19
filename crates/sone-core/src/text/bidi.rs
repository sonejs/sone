#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BaseDir {
    Ltr,
    Rtl,
    Auto,
}

fn is_rtl(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x05FF | 0x0600..=0x06FF | 0x0700..=0x074F | 0x0750..=0x077F
        | 0x07C0..=0x07FF | 0x0800..=0x085F | 0x0860..=0x086F | 0x08A0..=0x08FF
        | 0xFB1D..=0xFB4F | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF)
}

pub fn has_rtl_text(text: &str) -> bool {
    text.chars().any(is_rtl)
}

fn is_strong_ltr(cp: u32) -> bool {
    matches!(cp,
        0x0041..=0x005A | 0x0061..=0x007A | 0x00C0..=0x02B8 | 0x0370..=0x03FF
        | 0x0400..=0x04FF | 0x0530..=0x058F | 0x10A0..=0x10FF | 0x1100..=0x11FF
        | 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF)
}

/// UBA P2–P3 first-strong heuristic.
pub fn detect_paragraph_direction(text: &str) -> Dir {
    for c in text.chars() {
        if is_rtl(c) {
            return Dir::Rtl;
        }
        if is_strong_ltr(c as u32) {
            return Dir::Ltr;
        }
    }
    Dir::Ltr
}

pub fn resolve_paragraph_direction(text: &str, base: Option<BaseDir>) -> Dir {
    match base {
        Some(BaseDir::Rtl) => Dir::Rtl,
        Some(BaseDir::Ltr) => Dir::Ltr,
        Some(BaseDir::Auto) => detect_paragraph_direction(text),
        None => Dir::Ltr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_rtl() {
        assert!(!has_rtl_text(""));
        assert!(!has_rtl_text("Hello World"));
        assert!(!has_rtl_text("1234567890 !@#$%"));
        assert!(has_rtl_text("مرحبا بالعالم"));
        assert!(has_rtl_text("שָׁלוֹם"));
        assert!(has_rtl_text("Hello مرحبا"));
        assert!(has_rtl_text("\u{0710}\u{0712}\u{0713}"));
    }

    #[test]
    fn first_strong() {
        assert_eq!(detect_paragraph_direction(""), Dir::Ltr);
        assert_eq!(detect_paragraph_direction("Hello"), Dir::Ltr);
        assert_eq!(detect_paragraph_direction("مرحبا"), Dir::Rtl);
        assert_eq!(detect_paragraph_direction("123 مرحبا"), Dir::Rtl);
        assert_eq!(detect_paragraph_direction("!!! Hello مرحبا"), Dir::Ltr);
        assert_eq!(detect_paragraph_direction("!@#$"), Dir::Ltr);
    }

    #[test]
    fn resolve() {
        assert_eq!(
            resolve_paragraph_direction("مرحبا", Some(BaseDir::Ltr)),
            Dir::Ltr
        );
        assert_eq!(
            resolve_paragraph_direction("Hello", Some(BaseDir::Rtl)),
            Dir::Rtl
        );
        assert_eq!(
            resolve_paragraph_direction("مرحبا", Some(BaseDir::Auto)),
            Dir::Rtl
        );
        assert_eq!(resolve_paragraph_direction("مرحبا", None), Dir::Ltr);
    }
}
