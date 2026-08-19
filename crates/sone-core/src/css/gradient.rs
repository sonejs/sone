use super::color::{parse_color, Color};
use super::num::parse_float;

#[derive(Debug, Clone, PartialEq)]
pub enum GradientKind {
    Linear,
    RepeatingLinear,
    Radial,
    RepeatingRadial,
    Conic,
    RepeatingConic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Orientation {
    Directional(String),
    /// Angle in degrees.
    Angular(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopLength {
    Percent(f64),
    Px(f64),
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    /// Re-serialized color text, fed to `parse_color`.
    pub color: String,
    pub length: Option<StopLength>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub kind: GradientKind,
    pub orientation: Option<Orientation>,
    pub stops: Vec<ColorStop>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("{0}")]
pub struct GradientError(pub String);

struct Scanner<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Scanner<'a> {
    fn rest(&self) -> &'a str {
        &self.s[self.i..]
    }
    fn skip_ws(&mut self) {
        while self.rest().starts_with(|c: char| c.is_whitespace()) {
            self.i += self.rest().chars().next().unwrap().len_utf8();
        }
    }
    fn eat(&mut self, tok: &str) -> bool {
        self.skip_ws();
        if self.rest().starts_with(tok) {
            self.i += tok.len();
            true
        } else {
            false
        }
    }
    fn eat_ci(&mut self, tok: &str) -> bool {
        self.skip_ws();
        let r = self.rest();
        if r.len() >= tok.len() && r[..tok.len()].eq_ignore_ascii_case(tok) {
            self.i += tok.len();
            true
        } else {
            false
        }
    }
    fn peek_ci(&mut self, tok: &str) -> bool {
        self.skip_ws();
        let r = self.rest();
        r.len() >= tok.len() && r[..tok.len()].eq_ignore_ascii_case(tok)
    }
    fn eof(&mut self) -> bool {
        self.skip_ws();
        self.rest().is_empty()
    }

    /// `(-?(([0-9]*\.[0-9]+)|([0-9]+\.?)))`
    fn number(&mut self) -> Option<f64> {
        self.skip_ws();
        let r = self.rest().as_bytes();
        let mut j = 0;
        if j < r.len() && r[j] == b'-' {
            j += 1;
        }
        let ds = j;
        while j < r.len() && r[j].is_ascii_digit() {
            j += 1;
        }
        let int_digits = j - ds;
        if j < r.len() && r[j] == b'.' {
            j += 1;
            let fs = j;
            while j < r.len() && r[j].is_ascii_digit() {
                j += 1;
            }
            if j == fs && int_digits == 0 {
                return None;
            }
        } else if int_digits == 0 {
            return None;
        }
        let text = &self.rest()[..j];
        self.i += j;
        Some(parse_float(text))
    }

    fn number_with_unit(&mut self, unit: &str) -> Option<f64> {
        let save = self.i;
        match self.number() {
            Some(n) if self.rest().starts_with(unit) => {
                self.i += unit.len();
                Some(n)
            }
            _ => {
                self.i = save;
                None
            }
        }
    }

    fn ident(&mut self) -> Option<String> {
        self.skip_ws();
        let end = self
            .rest()
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(self.rest().len());
        if end == 0 {
            return None;
        }
        let s = self.rest()[..end].to_string();
        self.i += end;
        Some(s)
    }
}

const POSITION_KEYWORDS: &[&str] = &["left", "center", "right", "top", "bottom"];

fn match_angle(sc: &mut Scanner) -> Option<f64> {
    if let Some(v) = sc.number_with_unit("deg") {
        return Some(v);
    }
    if let Some(v) = sc.number_with_unit("rad") {
        return Some(v.to_degrees());
    }
    if let Some(v) = sc.number_with_unit("grad") {
        return Some(v * 0.9);
    }
    if let Some(v) = sc.number_with_unit("turn") {
        return Some(v * 360.0);
    }
    None
}

fn match_side_or_corner(sc: &mut Scanner) -> Option<String> {
    let save = sc.i;
    if !sc.eat_ci("to ") {
        sc.i = save;
        return None;
    }
    let first = sc.ident()?.to_lowercase();
    if !matches!(first.as_str(), "left" | "right" | "top" | "bottom") {
        sc.i = save;
        return None;
    }
    let save2 = sc.i;
    if let Some(second) = sc.ident() {
        let second = second.to_lowercase();
        let ok = match first.as_str() {
            "left" | "right" => matches!(second.as_str(), "top" | "bottom"),
            _ => matches!(second.as_str(), "left" | "right"),
        };
        if ok {
            return Some(format!("{first} {second}"));
        }
    }
    sc.i = save2;
    Some(first)
}

fn match_linear_orientation(sc: &mut Scanner) -> Option<Orientation> {
    if let Some(d) = match_side_or_corner(sc) {
        return Some(Orientation::Directional(d));
    }
    let save = sc.i;
    if let Some(id) = sc.ident() {
        let low = id.to_lowercase();
        if POSITION_KEYWORDS.contains(&low.as_str()) {
            return Some(Orientation::Directional(low));
        }
    }
    sc.i = save;
    match_angle(sc).map(Orientation::Angular)
}

fn match_distance(sc: &mut Scanner) -> Option<StopLength> {
    if let Some(v) = sc.number_with_unit("%") {
        return Some(StopLength::Percent(v));
    }
    let save = sc.i;
    if let Some(id) = sc.ident() {
        let low = id.to_lowercase();
        if POSITION_KEYWORDS.contains(&low.as_str()) {
            return Some(StopLength::Keyword(low));
        }
    }
    sc.i = save;
    for unit in ["px", "em", "rem", "vw", "vh", "vmin", "vmax", "ch", "ex"] {
        if let Some(v) = sc.number_with_unit(unit) {
            // Non-px absolute units are treated as pixels, matching the TS parser.
            return Some(StopLength::Px(v));
        }
    }
    None
}

fn match_color(sc: &mut Scanner) -> Option<String> {
    sc.skip_ws();
    if sc.rest().starts_with('#') {
        let hex: String = sc.rest()[1..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect();
        let n = hex.len();
        let take = if n >= 8 {
            8
        } else if n >= 6 {
            6
        } else if n >= 4 {
            4
        } else if n >= 3 {
            3
        } else {
            return None;
        };
        sc.i += 1 + take;
        return Some(format!("#{}", &hex[..take]));
    }
    for (name, arity_fn) in [("hsla", true), ("hsl", true), ("rgba", true), ("rgb", true)] {
        let _ = arity_fn;
        if sc.peek_ci(name) {
            let save = sc.i;
            sc.eat_ci(name);
            if !sc.eat("(") {
                sc.i = save;
                continue;
            }
            let start = sc.i;
            let mut depth = 1;
            while sc.i < sc.s.len() && depth > 0 {
                let c = sc.s[sc.i..].chars().next().unwrap();
                if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                }
                sc.i += c.len_utf8();
            }
            if depth > 0 {
                sc.i = save;
                continue;
            }
            let inner = &sc.s[start..sc.i - 1];
            return Some(format!("{name}({inner})"));
        }
    }
    let save = sc.i;
    if let Some(id) = sc.ident() {
        if !id.is_empty() {
            return Some(id);
        }
    }
    sc.i = save;
    None
}

fn match_color_stop(sc: &mut Scanner) -> Result<ColorStop, GradientError> {
    let color = match_color(sc).ok_or_else(|| GradientError("Expected color definition".into()))?;
    let length = match_distance(sc);
    if length.is_some() {
        let _second = match_distance(sc);
    }
    Ok(ColorStop { color, length })
}

fn match_gradient(sc: &mut Scanner) -> Result<Option<Gradient>, GradientError> {
    sc.skip_ws();
    // optional vendor prefix
    for p in ["-webkit-", "-o-", "-ms-", "-moz-"] {
        if sc.peek_ci(p) {
            sc.eat_ci(p);
            break;
        }
    }
    let kinds = [
        ("repeating-linear-gradient", GradientKind::RepeatingLinear),
        ("linear-gradient", GradientKind::Linear),
        ("repeating-radial-gradient", GradientKind::RepeatingRadial),
        ("radial-gradient", GradientKind::Radial),
        ("repeating-conic-gradient", GradientKind::RepeatingConic),
        ("conic-gradient", GradientKind::Conic),
    ];
    let mut kind = None;
    for (name, k) in kinds {
        if sc.peek_ci(name) {
            sc.eat_ci(name);
            kind = Some(k);
            break;
        }
    }
    let Some(kind) = kind else { return Ok(None) };
    if !sc.eat("(") {
        return Err(GradientError("Missing (".into()));
    }

    let orientation = match kind {
        GradientKind::Linear | GradientKind::RepeatingLinear => {
            let o = match_linear_orientation(sc);
            if o.is_some() && !sc.eat(",") {
                return Err(GradientError("Missing comma before color stops".into()));
            }
            o
        }
        _ => {
            // Radial/conic prelude: consume everything before the first comma
            // that is followed by a color. sone ignores the shape metadata.
            let save = sc.i;
            let mut consumed = false;
            loop {
                let before = sc.i;
                if match_distance(sc).is_some() || sc.ident().is_some() {
                    if sc.i == before {
                        break;
                    }
                    consumed = true;
                    // "at <pos>" and shape keywords are all idents/distances
                    sc.skip_ws();
                    if sc.rest().starts_with(',') || sc.rest().starts_with(')') {
                        break;
                    }
                } else {
                    break;
                }
            }
            if consumed && sc.rest().starts_with(',') {
                sc.eat(",");
            } else {
                sc.i = save;
            }
            None
        }
    };

    let mut stops = Vec::new();
    loop {
        sc.skip_ws();
        if sc.rest().starts_with(')') {
            break;
        }
        stops.push(match_color_stop(sc)?);
        if !sc.eat(",") {
            break;
        }
    }
    if !sc.eat(")") {
        return Err(GradientError("Missing )".into()));
    }
    Ok(Some(Gradient {
        kind,
        orientation,
        stops,
    }))
}

/// Port of `gradient-parser`'s `parse` for the grammar sone accepts.
pub fn parse_gradients(input: &str) -> Result<Vec<Gradient>, GradientError> {
    let mut sc = Scanner { s: input, i: 0 };
    let mut out = Vec::new();
    while let Some(g) = match_gradient(&mut sc)? {
        out.push(g);
        if !sc.eat(",") {
            break;
        }
    }
    if !sc.eof() {
        return Err(GradientError("Invalid input not EOF".into()));
    }
    Ok(out)
}

// ── isColor ────────────────────────────────────────────────────────────────

fn all_digits_in(s: &str, min: usize, max: usize) -> bool {
    let n = s.len();
    n >= min && n <= max && s.bytes().all(|b| b.is_ascii_digit())
}

fn is_decimal(s: &str) -> bool {
    // /\d*(?:\.\d+)?/ fully matched, non-empty
    if s.is_empty() {
        return false;
    }
    let (int, frac) = match s.split_once('.') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    };
    if !int.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    match frac {
        Some(f) => !f.is_empty() && f.bytes().all(|b| b.is_ascii_digit()),
        None => !int.is_empty(),
    }
}

fn is_percent_decimal(s: &str) -> bool {
    s.strip_suffix('%').map(is_decimal).unwrap_or(false)
}

/// Mirrors the regex battery in `src/gradient.ts`. Case-sensitive for names,
/// deliberately narrower than `parse_color`.
pub fn is_color(value: &str) -> bool {
    if value == "transparent" {
        return true;
    }
    if super::named::named(value).is_some() && value.chars().all(|c| c.is_ascii_lowercase()) {
        return true;
    }
    // ^rgb\((\d{1,3}),\s*(\d{1,3}),\s*(\d{1,3})\)$
    if let Some(inner) = value.strip_prefix("rgb(").and_then(|v| v.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3
            && all_digits_in(parts[0], 1, 3)
            && parts[1].starts_with(' ') == parts[1].starts_with(' ')
            && all_digits_in(parts[1].trim_start_matches(' '), 1, 3)
            && all_digits_in(parts[2].trim_start_matches(' '), 1, 3)
            && parts[1].trim_start_matches(' ').len() + count_leading_spaces(parts[1])
                == parts[1].len()
            && parts[2].trim_start_matches(' ').len() + count_leading_spaces(parts[2])
                == parts[2].len()
        {
            return true;
        }
    }
    if let Some(inner) = value
        .strip_prefix("rgba(")
        .and_then(|v| v.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4
            && all_digits_in(parts[0], 1, 3)
            && all_digits_in(parts[1].trim_start_matches(' '), 1, 3)
            && all_digits_in(parts[2].trim_start_matches(' '), 1, 3)
            && is_decimal(parts[3].trim_start_matches(' '))
        {
            return true;
        }
    }
    // ^hsl\(\s*(\d+)\s*,\s*(\d*(?:\.\d+)?%)\s*,\s*(\d*(?:\.\d+)?%)\)$
    if let Some(inner) = value.strip_prefix("hsl(").and_then(|v| v.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3
            && !parts[0].trim().is_empty()
            && parts[0].trim().bytes().all(|b| b.is_ascii_digit())
            && is_percent_decimal(parts[1].trim())
            && is_percent_decimal(parts[2].trim_start_matches(' '))
        {
            return true;
        }
    }
    if let Some(inner) = value
        .strip_prefix("hsla(")
        .and_then(|v| v.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4
            && !parts[0].is_empty()
            && parts[0].bytes().all(|b| b.is_ascii_digit())
            && is_percent_decimal(parts[1].trim_start_matches(' '))
            && is_percent_decimal(parts[2].trim_start_matches(' '))
            && is_decimal(parts[3].trim_start_matches(' '))
        {
            return true;
        }
    }
    // ^#([a-f0-9]{3,4}|[a-f0-9]{4}(?:[a-f0-9]{2}){1,2})\b$
    if let Some(hex) = value.strip_prefix('#') {
        if hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return matches!(hex.len(), 3 | 4 | 6 | 8);
        }
    }
    false
}

fn count_leading_spaces(s: &str) -> usize {
    s.bytes().take_while(|b| *b == b' ').count()
}

// ── generateGradient ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedGradient {
    pub start: Vec2,
    pub end: Vec2,
    pub colors: Vec<Color>,
    pub locations: Vec<f64>,
}

fn pixels_for_color(
    stop: &ColorStop,
    stops_len: usize,
    index: usize,
    max_width: Option<f64>,
) -> f64 {
    match &stop.length {
        None => {
            if stops_len <= 1 {
                0.0
            } else {
                (1.0 / (stops_len as f64 - 1.0)) * index as f64
            }
        }
        Some(StopLength::Px(v)) => *v,
        Some(StopLength::Percent(v)) => match max_width {
            Some(w) if w != 0.0 => (v * w) / 100.0,
            _ => v / 100.0,
        },
        Some(StopLength::Keyword(_)) => 0.0,
    }
}

fn colors_and_locations(stops: &[ColorStop], max_width: Option<f64>) -> (Vec<Color>, Vec<f64>) {
    let mut colors = Vec::with_capacity(stops.len());
    let mut locations = Vec::with_capacity(stops.len());
    for (i, s) in stops.iter().enumerate() {
        colors.push(parse_color(&s.color));
        locations.push(pixels_for_color(s, stops.len(), i, max_width));
    }
    (colors, locations)
}

fn repeating_colors_and_locations(stops: &[ColorStop], width: f64) -> (Vec<Color>, Vec<f64>) {
    let (initial_colors, initial_locations) = colors_and_locations(stops, Some(width));
    let Some(&max_value) = initial_locations.last() else {
        return (vec![], vec![]);
    };
    if max_value == 0.0 || !max_value.is_finite() {
        return (vec![], vec![]);
    }
    let increment = max_value / width;
    let max_chunks = (width / max_value).round() as i64;

    let mut locations = Vec::new();
    for i in 0..max_chunks.max(0) {
        for j in &initial_locations {
            locations.push(j / width + increment * i as f64);
        }
    }
    let colors = (0..locations.len())
        .map(|i| initial_colors[i % initial_colors.len().max(1)])
        .collect();
    (colors, locations)
}

fn round4(n: f64) -> f64 {
    (n * 10000.0).round() / 10000.0
}

fn vectors_by_angle(alfa: f64) -> (Vec2, Vec2) {
    let angle = alfa.to_radians();
    let len = round4(angle.sin().abs() + angle.cos().abs());
    let (cx, cy) = (0.5, 0.5);
    let y_diff = ((angle - std::f64::consts::FRAC_PI_2).sin() * len) / 2.0;
    let x_diff = ((angle - std::f64::consts::FRAC_PI_2).cos() * len) / 2.0;
    (
        Vec2 {
            x: cx - x_diff,
            y: cy - y_diff,
        },
        Vec2 {
            x: cx + x_diff,
            y: cy + y_diff,
        },
    )
}

fn vectors_by_direction(direction: &str) -> Option<(Vec2, Vec2)> {
    let deg = match direction {
        "top" => 0.0,
        "right" => 90.0,
        "bottom" => 180.0,
        "left" => 270.0,
        "left top" => 315.0,
        "left bottom" => 225.0,
        "right top" => 45.0,
        "right bottom" => 135.0,
        _ => return None,
    };
    Some(vectors_by_angle(deg))
}

fn vectors_by_orientation(o: Option<&Orientation>) -> Option<(Vec2, Vec2)> {
    match o {
        None => Some(vectors_by_angle(180.0)),
        Some(Orientation::Directional(d)) => vectors_by_direction(d),
        Some(Orientation::Angular(a)) => Some(vectors_by_angle(*a)),
    }
}

/// Linear (and repeating-linear) gradients resolved to Skia inputs.
/// Radial kinds are handled separately by the backend.
pub fn generate_gradient(gradients: &[Gradient], width: f64, height: f64) -> Vec<ResolvedGradient> {
    let _ = height;
    let mut out = Vec::new();
    for g in gradients {
        let (colors, locations) = match g.kind {
            GradientKind::Linear => colors_and_locations(&g.stops, None),
            GradientKind::RepeatingLinear => repeating_colors_and_locations(&g.stops, width),
            _ => continue,
        };
        let Some((start, end)) = vectors_by_orientation(g.orientation.as_ref()) else {
            continue;
        };
        out.push(ResolvedGradient {
            start,
            end,
            colors,
            locations,
        });
    }
    out
}

/// Clamp to [0,1] and force monotonically increasing stops, as Skia requires.
pub fn normalize_stops(locations: &[f64]) -> Vec<f32> {
    let mut stops: Vec<f32> = locations
        .iter()
        .map(|l| (*l).clamp(0.0, 1.0) as f32)
        .collect();
    for i in 1..stops.len() {
        if stops[i] < stops[i - 1] {
            stops[i] = stops[i - 1];
        }
    }
    stops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_color_accepts() {
        for v in [
            "#ff0000",
            "red",
            "rgb(255, 0, 0)",
            "rgba(255, 0, 0, 0.5)",
            "hsl(0, 100%, 50%)",
            "hsla(0, 100%, 50%, 0.5)",
            "transparent",
            "#fff",
            "#ffff",
            "#ffffff",
            "#ffffffff",
            "rgb(255,255,255)",
        ] {
            assert!(is_color(v), "{v} should be a color");
        }
    }

    #[test]
    fn is_color_rejects() {
        for v in [
            "not-a-color",
            "",
            "123",
            "linear-gradient(red, blue)",
            "rgba(255, 255, 255)",
            "#gggggg",
            "#ff",
            "RED",
            "Blue",
            "invalidcolor",
            "redd",
        ] {
            assert!(!is_color(v), "{v} should not be a color");
        }
    }

    #[test]
    fn parses_linear() {
        let g = parse_gradients("linear-gradient(90deg, red 0%, blue 100%)").unwrap();
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].kind, GradientKind::Linear);
        assert_eq!(g[0].orientation, Some(Orientation::Angular(90.0)));
        assert_eq!(g[0].stops.len(), 2);
        assert_eq!(g[0].stops[0].color, "red");
        assert_eq!(g[0].stops[0].length, Some(StopLength::Percent(0.0)));
    }

    #[test]
    fn parses_side_or_corner() {
        let g = parse_gradients("linear-gradient(to right bottom, #fff, #000)").unwrap();
        assert_eq!(
            g[0].orientation,
            Some(Orientation::Directional("right bottom".into()))
        );
        assert_eq!(g[0].stops[0].color, "#fff");
    }

    #[test]
    fn parses_functional_stops() {
        let g =
            parse_gradients("linear-gradient(rgba(0,0,0,0.5), hsl(120, 50%, 50%) 40%)").unwrap();
        assert_eq!(g[0].stops[0].color, "rgba(0,0,0,0.5)");
        assert_eq!(g[0].stops[1].color, "hsl(120, 50%, 50%)");
        assert_eq!(g[0].stops[1].length, Some(StopLength::Percent(40.0)));
    }

    #[test]
    fn parses_multiple_gradients() {
        let g =
            parse_gradients("linear-gradient(red, blue), linear-gradient(green, black)").unwrap();
        assert_eq!(g.len(), 2);
    }

    #[test]
    fn parses_radial() {
        let g = parse_gradients("radial-gradient(circle at center, red, blue)").unwrap();
        assert_eq!(g[0].kind, GradientKind::Radial);
        assert_eq!(g[0].stops.len(), 2);
    }

    #[test]
    fn default_stop_positions() {
        let g = parse_gradients("linear-gradient(red, green, blue)").unwrap();
        let r = generate_gradient(&g, 100.0, 100.0);
        assert_eq!(r[0].locations, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn angle_vectors() {
        let (s, e) = vectors_by_angle(180.0);
        assert!((s.x - 0.5).abs() < 1e-6 && (s.y - 0.0).abs() < 1e-6);
        assert!((e.x - 0.5).abs() < 1e-6 && (e.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn radial_skipped_by_linear_generator() {
        let g = parse_gradients("radial-gradient(red, blue)").unwrap();
        assert!(generate_gradient(&g, 100.0, 100.0).is_empty());
    }

    #[test]
    fn monotonic_stops() {
        assert_eq!(
            normalize_stops(&[0.0, 0.8, 0.4, 1.2]),
            vec![0.0, 0.8, 0.8, 1.0]
        );
    }
}
