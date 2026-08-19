use super::color::{parse_color, Color};
use super::num::parse_float;

#[derive(Debug, Clone, PartialEq)]
pub enum FilterOp {
    /// Gaussian sigma (already halved from the CSS radius).
    Blur(f64),
    /// 4x5 row-major color matrix.
    ColorMatrix([f32; 20]),
    DropShadow {
        dx: f64,
        dy: f64,
        sigma: f64,
        color: Color,
    },
}

fn parse_value(raw: &str) -> f64 {
    let v = raw.trim();
    if let Some(p) = v.strip_suffix('%') {
        return parse_float(p) / 100.0;
    }
    parse_float(v)
}

fn radius_to_sigma(radius: f64) -> f64 {
    radius / 2.0
}

pub fn saturate_matrix(amount: f32) -> [f32; 20] {
    let (r, g, b) = (0.213f32, 0.715f32, 0.072f32);
    let s = amount;
    [
        r + s * (1.0 - r),
        g - s * g,
        b - s * b,
        0.0,
        0.0,
        r - s * r,
        g + s * (1.0 - g),
        b - s * b,
        0.0,
        0.0,
        r - s * r,
        g - s * g,
        b + s * (1.0 - b),
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    ]
}

pub fn sepia_matrix(a: f32) -> [f32; 20] {
    [
        0.393 + 0.607 * (1.0 - a),
        0.769 - 0.769 * (1.0 - a),
        0.189 - 0.189 * (1.0 - a),
        0.0,
        0.0,
        0.349 - 0.349 * (1.0 - a),
        0.686 + 0.314 * (1.0 - a),
        0.168 - 0.168 * (1.0 - a),
        0.0,
        0.0,
        0.272 - 0.272 * (1.0 - a),
        0.534 - 0.534 * (1.0 - a),
        0.131 + 0.869 * (1.0 - a),
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    ]
}

pub fn grayscale_matrix(a: f32) -> [f32; 20] {
    saturate_matrix(1.0 - a)
}

pub fn scale_matrix(a: f32) -> [f32; 20] {
    [
        a, 0.0, 0.0, 0.0, 0.0, 0.0, a, 0.0, 0.0, 0.0, 0.0, 0.0, a, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        0.0,
    ]
}

pub fn contrast_matrix(a: f32) -> [f32; 20] {
    let i = 0.5 * (1.0 - a);
    [
        a, 0.0, 0.0, 0.0, i, 0.0, a, 0.0, 0.0, i, 0.0, 0.0, a, 0.0, i, 0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

pub fn invert_matrix(a: f32) -> [f32; 20] {
    let s = 1.0 - 2.0 * a;
    [
        s, 0.0, 0.0, 0.0, a, 0.0, s, 0.0, 0.0, a, 0.0, 0.0, s, 0.0, a, 0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

pub fn opacity_matrix(a: f32) -> [f32; 20] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        a, 0.0,
    ]
}

pub fn hue_rotate_matrix(degrees: f32) -> [f32; 20] {
    let rad = degrees.to_radians();
    let (c, s) = (rad.cos(), rad.sin());
    [
        0.213 + c * 0.787 - s * 0.213,
        0.715 - c * 0.715 - s * 0.715,
        0.072 - c * 0.072 + s * 0.928,
        0.0,
        0.0,
        0.213 - c * 0.213 + s * 0.143,
        0.715 + c * 0.285 + s * 0.14,
        0.072 - c * 0.072 - s * 0.283,
        0.0,
        0.0,
        0.213 - c * 0.213 - s * 0.787,
        0.715 - c * 0.715 + s * 0.715,
        0.072 + c * 0.928 + s * 0.072,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    ]
}

fn split_functions(input: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let b: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_alphabetic() || b[i] == '-' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphabetic() || b[i] == '-') {
                i += 1;
            }
            if i < b.len() && b[i] == '(' {
                let name: String = b[start..i].iter().collect::<String>().to_lowercase();
                i += 1;
                let astart = i;
                while i < b.len() && b[i] != ')' {
                    i += 1;
                }
                if i >= b.len() {
                    break;
                }
                let args: String = b[astart..i].iter().collect();
                i += 1;
                out.push((name, args));
                continue;
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Parse CSS filter strings into an ordered op list. Unknown functions are
/// reported so the caller can warn once.
pub fn parse_css_filter(filters: &[String]) -> (Vec<FilterOp>, Vec<String>) {
    let mut ops = Vec::new();
    let mut unknown = Vec::new();

    for entry in filters {
        for (name, args) in split_functions(entry) {
            match name.as_str() {
                "blur" => {
                    let sigma = radius_to_sigma(parse_value(&args));
                    if sigma > 0.0 {
                        ops.push(FilterOp::Blur(sigma));
                    }
                }
                "brightness" => ops.push(FilterOp::ColorMatrix(scale_matrix(
                    parse_value(&args) as f32
                ))),
                "contrast" => ops.push(FilterOp::ColorMatrix(contrast_matrix(
                    parse_value(&args) as f32
                ))),
                "saturate" => ops.push(FilterOp::ColorMatrix(saturate_matrix(
                    parse_value(&args) as f32
                ))),
                "grayscale" => ops.push(FilterOp::ColorMatrix(grayscale_matrix(
                    parse_value(&args) as f32,
                ))),
                "sepia" => ops.push(FilterOp::ColorMatrix(sepia_matrix(
                    parse_value(&args) as f32
                ))),
                "invert" => ops.push(FilterOp::ColorMatrix(invert_matrix(
                    parse_value(&args) as f32
                ))),
                "opacity" => ops.push(FilterOp::ColorMatrix(opacity_matrix(
                    parse_value(&args) as f32
                ))),
                "hue-rotate" => ops.push(FilterOp::ColorMatrix(hue_rotate_matrix(
                    parse_float(&args) as f32,
                ))),
                "drop-shadow" => {
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    let dx = parse_value(parts.first().copied().unwrap_or("0"));
                    let dy = parse_value(parts.get(1).copied().unwrap_or("0"));
                    let sigma = radius_to_sigma(parse_value(parts.get(2).copied().unwrap_or("0")));
                    let color_str = if parts.len() > 3 {
                        parts[3..].join(" ")
                    } else {
                        "black".into()
                    };
                    ops.push(FilterOp::DropShadow {
                        dx,
                        dy,
                        sigma,
                        color: parse_color(&color_str),
                    });
                }
                "none" => {}
                other => unknown.push(other.to_string()),
            }
        }
    }
    (ops, unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_and_parses() {
        let (ops, unknown) = parse_css_filter(&["blur(4px) brightness(0.5)".into()]);
        assert!(unknown.is_empty());
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], FilterOp::Blur(2.0));
    }

    #[test]
    fn zero_blur_dropped() {
        let (ops, _) = parse_css_filter(&["blur(0px)".into()]);
        assert!(ops.is_empty());
    }

    #[test]
    fn percent_values() {
        let (ops, _) = parse_css_filter(&["grayscale(50%)".into()]);
        match &ops[0] {
            FilterOp::ColorMatrix(m) => assert!((m[0] - saturate_matrix(0.5)[0]).abs() < 1e-6),
            _ => panic!(),
        }
    }

    #[test]
    fn drop_shadow_args() {
        let (ops, _) = parse_css_filter(&["drop-shadow(2px 3px 4px red)".into()]);
        match &ops[0] {
            FilterOp::DropShadow {
                dx,
                dy,
                sigma,
                color,
            } => {
                assert_eq!((*dx, *dy, *sigma), (2.0, 3.0, 2.0));
                assert_eq!(color.r(), 1.0);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn unknown_reported() {
        let (_, unknown) = parse_css_filter(&["wobble(3)".into()]);
        assert_eq!(unknown, vec!["wobble"]);
    }
}
