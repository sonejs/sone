/// `Number.parseFloat` — lenient leading-numeric prefix parse.
pub fn parse_float(s: &str) -> f64 {
    let t = s.trim_start();
    let b = t.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    if t[i..].starts_with("Infinity") {
        let v = f64::INFINITY;
        return if t.starts_with('-') { -v } else { v };
    }
    let start_digits = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == start_digits || (i == start_digits + 1 && b[start_digits] == b'.') {
        return f64::NAN;
    }
    let mant_end = i;
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        let ds = j;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        if j > ds {
            i = j;
        } else {
            i = mant_end;
        }
    }
    t[..i].parse().unwrap_or(f64::NAN)
}

/// `Number(...)` — strict whole-string numeric conversion.
pub fn js_number(s: &str) -> f64 {
    let t = s.trim();
    if t.is_empty() {
        return 0.0;
    }
    if t == "Infinity" || t == "+Infinity" {
        return f64::INFINITY;
    }
    if t == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16)
            .map(|v| v as f64)
            .unwrap_or(f64::NAN);
    }
    t.parse().unwrap_or(f64::NAN)
}

/// `Number.parseInt(s, radix)`.
pub fn parse_int_radix(s: &str, radix: u32) -> f64 {
    let t = s.trim_start();
    let (neg, t) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let end = t
        .char_indices()
        .find(|(_, c)| !c.is_digit(radix))
        .map(|(i, _)| i)
        .unwrap_or(t.len());
    if end == 0 {
        return f64::NAN;
    }
    let v = u64::from_str_radix(&t[..end], radix)
        .map(|v| v as f64)
        .unwrap_or(f64::NAN);
    if neg {
        -v
    } else {
        v
    }
}
