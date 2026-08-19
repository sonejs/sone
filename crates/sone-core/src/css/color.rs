use super::num::{parse_float, parse_int_radix};

/// RGBA, 0..1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub [f32; 4]);

impl Default for Color {
    fn default() -> Self {
        BLACK
    }
}

pub const TRANSPARENT: Color = Color([0.0, 0.0, 0.0, 0.0]);
pub const BLACK: Color = Color([0.0, 0.0, 0.0, 1.0]);

impl Color {
    pub fn r(&self) -> f32 {
        self.0[0]
    }
    pub fn g(&self) -> f32 {
        self.0[1]
    }
    pub fn b(&self) -> f32 {
        self.0[2]
    }
    pub fn a(&self) -> f32 {
        self.0[3]
    }

    pub fn with_alpha(self, multiplier: f32) -> Color {
        if multiplier >= 1.0 {
            return self;
        }
        Color([
            self.0[0],
            self.0[1],
            self.0[2],
            clamp01(self.0[3] * multiplier),
        ])
    }
}

fn clamp01(n: f32) -> f32 {
    n.clamp(0.0, 1.0)
}

fn clamp01f(n: f64) -> f32 {
    clamp01(n as f32)
}

fn alpha(value: Option<&str>) -> f32 {
    let Some(v) = value else { return 1.0 };
    let t = v.trim();
    if let Some(p) = t.strip_suffix('%') {
        return clamp01f(parse_float(p) / 100.0);
    }
    let n = parse_float(t);
    if n.is_nan() {
        1.0
    } else {
        clamp01f(n)
    }
}

fn channel(value: &str) -> f32 {
    let t = value.trim();
    if let Some(p) = t.strip_suffix('%') {
        return clamp01f(parse_float(p) / 100.0);
    }
    let n = parse_float(t);
    if n.is_nan() {
        0.0
    } else {
        clamp01f(n / 255.0)
    }
}

fn hue_to_rgb(p: f64, q: f64, t: f64) -> f64 {
    let mut h = t;
    if h < 0.0 {
        h += 1.0
    }
    if h > 1.0 {
        h -= 1.0
    }
    if h < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * h;
    }
    if h < 1.0 / 2.0 {
        return q;
    }
    if h < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - h) * 6.0;
    }
    p
}

fn from_hsl(h: f64, s: f64, l: f64, a: f32) -> Color {
    let hue = (((h % 360.0) + 360.0) % 360.0) / 360.0;
    let sat = clamp01(s as f32) as f64;
    let lig = clamp01(l as f32) as f64;

    if sat == 0.0 {
        return Color([lig as f32, lig as f32, lig as f32, a]);
    }
    let q = if lig < 0.5 {
        lig * (1.0 + sat)
    } else {
        lig + sat - lig * sat
    };
    let p = 2.0 * lig - q;
    Color([
        hue_to_rgb(p, q, hue + 1.0 / 3.0) as f32,
        hue_to_rgb(p, q, hue) as f32,
        hue_to_rgb(p, q, hue - 1.0 / 3.0) as f32,
        a,
    ])
}

fn from_hex(hex: &str) -> Option<Color> {
    let h: Vec<char> = hex[1..].chars().collect();
    let expand = |c: char| (parse_int_radix(&format!("{c}{c}"), 16) / 255.0) as f32;
    let byte = |i: usize| {
        let s: String = h[i..i + 2].iter().collect();
        (parse_int_radix(&s, 16) / 255.0) as f32
    };

    match h.len() {
        3 | 4 => Some(Color([
            expand(h[0]),
            expand(h[1]),
            expand(h[2]),
            if h.len() == 4 { expand(h[3]) } else { 1.0 },
        ])),
        6 | 8 => Some(Color([
            byte(0),
            byte(2),
            byte(4),
            if h.len() == 8 { byte(6) } else { 1.0 },
        ])),
        _ => None,
    }
}

fn args(value: &str) -> Vec<String> {
    let Some(open) = value.find('(') else {
        return vec![];
    };
    let Some(close) = value.rfind(')') else {
        return vec![];
    };
    if close < open {
        return vec![];
    }
    value[open + 1..close]
        .replace('/', ",")
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Parse any CSS color sone accepts. Unrecognized input falls back to black.
pub fn parse_color(value: &str) -> Color {
    let key = value.trim().to_lowercase();

    if key == "transparent" || key == "none" {
        return TRANSPARENT;
    }
    if let Some(rgb) = super::named::named(&key) {
        return Color([
            ((rgb >> 16) & 0xff) as f32 / 255.0,
            ((rgb >> 8) & 0xff) as f32 / 255.0,
            (rgb & 0xff) as f32 / 255.0,
            1.0,
        ]);
    }
    if key.starts_with('#') {
        return from_hex(&key).unwrap_or(BLACK);
    }
    if key.starts_with("rgb") {
        let a = args(&key);
        if a.len() < 3 {
            return BLACK;
        }
        return Color([
            channel(&a[0]),
            channel(&a[1]),
            channel(&a[2]),
            alpha(a.get(3).map(|s| s.as_str())),
        ]);
    }
    if key.starts_with("hsl") {
        let a = args(&key);
        if a.len() < 3 {
            return BLACK;
        }
        return from_hsl(
            parse_float(&a[0]),
            parse_float(&a[1]) / 100.0,
            parse_float(&a[2]) / 100.0,
            alpha(a.get(3).map(|s| s.as_str())),
        );
    }
    BLACK
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(c: Color, e: [f32; 4]) {
        for i in 0..4 {
            assert!((c.0[i] - e[i]).abs() < 1e-4, "{:?} != {:?}", c.0, e);
        }
    }

    #[test]
    fn named_colors() {
        approx(parse_color("red"), [1.0, 0.0, 0.0, 1.0]);
        approx(parse_color("BLUE"), [0.0, 0.0, 1.0, 1.0]);
        approx(parse_color("green"), [0.0, 128.0 / 255.0, 0.0, 1.0]);
        approx(parse_color("white"), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn hex_forms() {
        approx(parse_color("#f00"), [1.0, 0.0, 0.0, 1.0]);
        approx(parse_color("#ff0000"), [1.0, 0.0, 0.0, 1.0]);
        approx(parse_color("#ff000080"), [1.0, 0.0, 0.0, 128.0 / 255.0]);
        approx(parse_color("#f008"), [1.0, 0.0, 0.0, 136.0 / 255.0]);
        approx(parse_color("#12345"), [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn functional_forms() {
        approx(parse_color("rgb(255, 0, 0)"), [1.0, 0.0, 0.0, 1.0]);
        approx(parse_color("rgba(255,0,0,0.5)"), [1.0, 0.0, 0.0, 0.5]);
        approx(parse_color("rgb(100%,0%,0%,50%)"), [1.0, 0.0, 0.0, 0.5]);
        // space-separated rgb() is not in the accepted grammar (matches TS)
        approx(parse_color("rgb(100% 0% 0% / 50%)"), [0.0, 0.0, 0.0, 1.0]);
        approx(parse_color("hsl(0, 100%, 50%)"), [1.0, 0.0, 0.0, 1.0]);
        approx(parse_color("hsl(120, 100%, 50%)"), [0.0, 1.0, 0.0, 1.0]);
        approx(parse_color("hsl(0, 0%, 50%)"), [0.5, 0.5, 0.5, 1.0]);
        approx(parse_color("hsl(-240, 100%, 50%)"), [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn fallbacks() {
        approx(parse_color("transparent"), [0.0, 0.0, 0.0, 0.0]);
        approx(parse_color("none"), [0.0, 0.0, 0.0, 0.0]);
        approx(parse_color("nonsense"), [0.0, 0.0, 0.0, 1.0]);
        approx(parse_color("rgb(1,2)"), [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn alpha_multiplier() {
        approx(parse_color("red").with_alpha(0.5), [1.0, 0.0, 0.0, 0.5]);
        approx(parse_color("red").with_alpha(1.0), [1.0, 0.0, 0.0, 1.0]);
    }
}
