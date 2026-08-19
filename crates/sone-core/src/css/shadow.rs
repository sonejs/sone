use super::num::{js_number, parse_float};

#[derive(Debug, Clone, PartialEq)]
pub struct CssShadow {
    pub inset: bool,
    pub offset_x: f64,
    pub offset_y: f64,
    pub blur_radius: f64,
    pub spread_radius: Option<f64>,
    pub color: Option<String>,
}

/// Split at separators that sit outside parentheses.
fn split_outside_parens(s: &str, is_sep: impl Fn(char) -> bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth = (depth - 1).max(0);
                cur.push(c);
            }
            _ if depth == 0 && is_sep(c) => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn is_length(v: &str) -> bool {
    if v == "0" {
        return true;
    }
    // /^[0-9]+[a-zA-Z%]+?$/
    let mut chars = v.chars().peekable();
    let mut digits = 0;
    while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
        chars.next();
        digits += 1;
    }
    if digits == 0 {
        return false;
    }
    let rest: String = chars.collect();
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphabetic() || c == '%')
}

fn to_num(v: &str) -> f64 {
    if v.ends_with("px") {
        let n = parse_float(v);
        if !n.is_nan() {
            return n;
        }
        return js_number(v);
    }
    if v == "0" {
        return 0.0;
    }
    js_number(v)
}

fn parse_value(s: &str) -> CssShadow {
    let parts: Vec<String> = split_outside_parens(s, |c| c.is_whitespace())
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();
    let inset = parts.iter().any(|p| p == "inset");
    let last = parts.last().cloned().unwrap_or_default();
    let color = if !is_length(&last) { Some(last) } else { None };

    let nums: Vec<f64> = parts
        .iter()
        .filter(|n| n.as_str() != "inset")
        .filter(|n| Some(n.as_str()) != color.as_deref())
        .map(|n| to_num(n))
        .collect();

    CssShadow {
        inset,
        offset_x: nums.first().copied().unwrap_or(f64::NAN),
        offset_y: nums.get(1).copied().unwrap_or(f64::NAN),
        blur_radius: nums.get(2).copied().unwrap_or(f64::NAN),
        spread_radius: nums.get(3).copied(),
        color,
    }
}

pub fn parse_shadow(s: &str) -> Vec<CssShadow> {
    split_outside_parens(s, |c| c == ',')
        .iter()
        .map(|v| parse_value(v.trim()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let r = parse_shadow("2px 4px 6px rgba(0,0,0,0.3)");
        assert_eq!(r.len(), 1);
        assert_eq!(
            r[0],
            CssShadow {
                inset: false,
                offset_x: 2.0,
                offset_y: 4.0,
                blur_radius: 6.0,
                spread_radius: None,
                color: Some("rgba(0,0,0,0.3)".into())
            }
        );
    }

    #[test]
    fn spread() {
        let r = parse_shadow("1px 2px 3px 4px red");
        assert_eq!(r[0].spread_radius, Some(4.0));
        assert_eq!(r[0].color.as_deref(), Some("red"));
    }

    #[test]
    fn inset() {
        let r = parse_shadow("inset 2px 4px 6px blue");
        assert!(r[0].inset);
        assert_eq!(r[0].offset_x, 2.0);
        assert_eq!(r[0].color.as_deref(), Some("blue"));
    }

    #[test]
    fn negatives() {
        let r = parse_shadow("-2px -4px 6px green");
        assert_eq!(r[0].offset_x, -2.0);
        assert_eq!(r[0].offset_y, -4.0);
        assert_eq!(r[0].color.as_deref(), Some("green"));
    }

    #[test]
    fn multiple() {
        let r = parse_shadow("2px 2px 4px red, -1px -1px 2px blue");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].color.as_deref(), Some("red"));
        assert_eq!(r[1].color.as_deref(), Some("blue"));
    }

    #[test]
    fn no_color() {
        let r = parse_shadow("2px 2px 4px");
        assert_eq!(r[0].color, None);
        assert_eq!(r[0].blur_radius, 4.0);
    }
}
