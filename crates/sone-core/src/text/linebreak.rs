use std::collections::HashSet;

const END_SYM: &str = "។៕)]?!»៖$រៗ%,;:";
const START_SYM: &str = "([«$#@";
const NBSP: char = '\u{00a0}';
const WORD_JOINER: char = '\u{2060}';
const KHMER_SUBSCRIPT: char = '\u{17d2}';

/// Characters the URL pattern refuses to consume.
fn is_url_stop(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '<' | '>' | '"' | '{' | '}' | '|' | '\\' | '^' | '`' | '[' | ']'
        )
}

/// End of the run of URL-legal characters starting at `at`.
fn url_run_end(text: &str, at: usize) -> usize {
    text[at..]
        .char_indices()
        .find(|(_, c)| is_url_stop(*c))
        .map(|(i, _)| at + i)
        .unwrap_or(text.len())
}

/// `www.` followed by a label and at least one dotted 2+ letter suffix.
fn is_bare_domain(run: &str) -> bool {
    let Some(rest) = run.strip_prefix("www.") else {
        return false;
    };
    let mut chars = rest.char_indices().peekable();

    match chars.next() {
        Some((_, c)) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    let mut label_len = 0;
    let mut last_alnum = true;
    while let Some(&(_, c)) = chars.peek() {
        if c.is_ascii_alphanumeric() || c == '-' {
            if label_len >= 62 {
                return false;
            }
            last_alnum = c.is_ascii_alphanumeric();
            label_len += 1;
            chars.next();
        } else {
            break;
        }
    }
    if !last_alnum {
        return false;
    }

    // One or more `.tld` groups of at least two letters.
    let mut suffixes = 0;
    while let Some(&(_, '.')) = chars.peek() {
        chars.next();
        let mut letters = 0;
        while let Some(&(_, c)) = chars.peek() {
            if c.is_ascii_alphabetic() {
                letters += 1;
                chars.next();
            } else {
                break;
            }
        }
        if letters < 2 {
            return false;
        }
        suffixes += 1;
    }
    suffixes > 0
}

/// Absolute `http`/`https`/`ftp` URLs and bare `www.` domains, leftmost-first
/// and non-overlapping — the same matches the TS `URL_PATTERN` produces.
fn find_urls(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        if !text.is_char_boundary(i) {
            i += 1;
            continue;
        }
        let rest = &text[i..];
        let scheme = ["https://", "http://", "ftp://"]
            .iter()
            .find(|s| rest.starts_with(**s));

        if let Some(scheme) = scheme {
            let end = url_run_end(text, i);
            // The pattern needs at least one character after the scheme.
            if end > i + scheme.len() {
                out.push((i, end));
                i = end;
                continue;
            }
        } else if rest.starts_with("www.") {
            let end = url_run_end(text, i);
            if is_bare_domain(&text[i..end]) {
                out.push((i, end));
                i = end;
                continue;
            }
        }

        i += rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
    }
    out
}

/// Digit, separator, digit — protects phone numbers and decimal groups.
fn find_phone_separators(text: &str) -> Vec<usize> {
    const SEPARATORS: [char; 6] = ['-', '.', '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}'];
    let mut out = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;
    while i + 2 < chars.len() {
        let (start, a) = chars[i];
        let (_, sep) = chars[i + 1];
        let (_, b) = chars[i + 2];
        if a.is_ascii_digit() && SEPARATORS.contains(&sep) && b.is_ascii_digit() {
            out.push(start);
            i += 3;
        } else {
            i += 1;
        }
    }
    out
}

fn is_inline_whitespace(c: char) -> bool {
    c.is_whitespace() && c != '\r' && c != '\n'
}

fn char_before(text: &str, i: usize) -> Option<char> {
    text[..i].chars().next_back()
}

fn prev_index(text: &str, i: usize) -> usize {
    match char_before(text, i) {
        Some(c) => i - c.len_utf8(),
        None => 0,
    }
}

fn collect_protected(text: &str) -> HashSet<usize> {
    let mut out = HashSet::new();

    for (i, c) in text.char_indices() {
        if c != NBSP && c != WORD_JOINER {
            continue;
        }
        let mut start = i;
        while start > 0 {
            let p = prev_index(text, start);
            match char_before(text, start) {
                Some(pc) if is_inline_whitespace(pc) => start = p,
                _ => break,
            }
        }
        let mut end = i + c.len_utf8();
        while end < text.len() {
            let nc = text[end..].chars().next().unwrap();
            if is_inline_whitespace(nc) {
                end += nc.len_utf8();
            } else {
                break;
            }
        }
        let mut b = start;
        while b < end {
            out.insert(b);
            b += text[b..].chars().next().unwrap().len_utf8();
        }
        out.insert(end);
    }

    for (s, e) in find_urls(text) {
        let mut b = s + text[s..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        while b < e {
            out.insert(b);
            b += text[b..].chars().next().unwrap().len_utf8();
        }
    }

    for s in find_phone_separators(text) {
        let digit_len = text[s..].chars().next().unwrap().len_utf8();
        let sep_len = text[s + digit_len..].chars().next().unwrap().len_utf8();
        out.insert(s + digit_len);
        out.insert(s + digit_len + sep_len);
    }

    out
}

/// Layer sone's custom rules over raw word-segment starts from the backend.
/// `segment_starts` must be ascending byte offsets, starting at 0.
pub fn apply_break_rules(text: &str, segment_starts: &[usize]) -> Vec<usize> {
    let protected = collect_protected(text);
    let end_sym: HashSet<char> = END_SYM.chars().collect();
    let start_sym: HashSet<char> = START_SYM.chars().collect();

    let mut out = Vec::with_capacity(segment_starts.len());
    for (n, &start) in segment_starts.iter().enumerate() {
        if start > text.len() {
            continue;
        }
        let end = segment_starts
            .get(n + 1)
            .copied()
            .unwrap_or(text.len())
            .min(text.len());
        let seg = &text[start..end];
        if seg.ends_with(KHMER_SUBSCRIPT) {
            continue;
        }
        if protected.contains(&start) {
            continue;
        }
        if let Some(next) = text[start..].chars().next() {
            if end_sym.contains(&next) {
                continue;
            }
        }
        if let Some(prev) = char_before(text, start) {
            if start_sym.contains(&prev) {
                continue;
            }
        }
        out.push(start);
    }
    out
}

/// UAX#29 word starts. Dictionary scripts (Khmer, Thai, Lao, Burmese) are not
/// segmented here — use [`word_starts`] for those.
pub fn uax29_word_starts(text: &str) -> Vec<usize> {
    use unicode_segmentation::UnicodeSegmentation;
    let mut out = Vec::new();
    for (i, _) in text.split_word_bound_indices() {
        out.push(i);
    }
    if out.is_empty() && !text.is_empty() {
        out.push(0);
    }
    out
}

/// Word-segment starts for scripts UAX#29 can segment on its own.
///
/// Dictionary scripts (Khmer, Thai, Lao, Burmese) have no spaces and no
/// UAX#29 word boundaries, so a backend that can reach a real dictionary —
/// [`crate::paint::TextEngine::word_starts`] — should override this.
pub fn word_starts(text: &str) -> Vec<usize> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut out = uax29_word_starts(text);
    if out.first() != Some(&0) {
        out.insert(0, 0);
    }
    out
}

/// True when `text` contains a script that needs dictionary segmentation.
pub fn needs_dictionary(text: &str) -> bool {
    text.chars().any(is_dictionary_script)
}

fn is_dictionary_script(c: char) -> bool {
    matches!(c as u32,
        0x1780..=0x17FF   // Khmer
        | 0x19E0..=0x19FF // Khmer symbols
        | 0x0E00..=0x0E7F // Thai
        | 0x0E80..=0x0EFF // Lao
        | 0x1000..=0x109F // Myanmar
    )
}

/// Split into maximal runs of dictionary-segmented script and everything else.
pub fn script_runs(text: &str) -> Vec<(usize, usize, bool)> {
    let mut runs: Vec<(usize, usize, bool)> = Vec::new();
    for (i, c) in text.char_indices() {
        let dictionary = is_dictionary_script(c);
        match runs.last_mut() {
            Some(last) if last.2 == dictionary => last.1 = i + c.len_utf8(),
            _ => runs.push((i, i + c.len_utf8(), dictionary)),
        }
    }
    runs
}

pub fn grapheme_starts(text: &str) -> Vec<usize> {
    use unicode_segmentation::UnicodeSegmentation;
    text.grapheme_indices(true).map(|(i, _)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brk(text: &str) -> Vec<usize> {
        apply_break_rules(text, &word_starts(text))
    }

    #[test]
    fn ordinary_spaces() {
        assert_eq!(brk("foo bar"), vec![0, 3, 4]);
    }

    #[test]
    fn word_joiner_glue() {
        assert_eq!(brk("foo\u{2060} bar"), vec![0]);
        assert_eq!(brk("foo \u{2060}bar"), vec![0]);
        assert_eq!(brk("foo \u{2060} bar"), vec![0]);
    }

    #[test]
    fn nbsp_glue() {
        assert_eq!(brk("foo\u{00a0}bar"), vec![0]);
        assert_eq!(brk("foo\u{00a0} bar"), vec![0]);
        assert_eq!(brk("foo \u{00a0}bar"), vec![0]);
        assert_eq!(brk("foo \u{00a0} bar"), vec![0]);
    }

    #[test]
    fn end_symbols() {
        assert_eq!(brk("foo,bar"), vec![0, 4]);
        assert_eq!(brk("foo;bar"), vec![0, 4]);
    }

    #[test]
    fn start_symbols() {
        assert_eq!(brk("foo #bar"), vec![0, 3, 4]);
        assert_eq!(brk("hello @user"), vec![0, 5, 6]);
    }

    #[test]
    fn urls() {
        assert_eq!(
            brk("visit https://example.com today"),
            vec![0, 5, 6, 25, 26]
        );
        assert_eq!(brk("see www.example.com here"), vec![0, 3, 4, 19, 20]);
        assert_eq!(brk("https://example.com/path?q=1"), vec![0]);
    }

    #[test]
    fn phone_numbers() {
        assert_eq!(brk("555-1234"), vec![0]);
        assert_eq!(brk("123.456"), vec![0]);
        assert_eq!(brk("555-123-4567"), vec![0]);
    }

    #[test]
    fn hyphenated_words_still_break() {
        assert_eq!(brk("well-known"), vec![0, 4, 5]);
        assert_eq!(brk("state-of-the-art"), vec![0, 5, 6, 8, 9, 12, 13]);
    }

    #[test]
    fn url_scanner_matches_the_regex_it_replaced() {
        assert_eq!(find_urls("visit https://example.com today"), vec![(6, 25)]);
        assert_eq!(find_urls("see www.example.com here"), vec![(4, 19)]);
        assert_eq!(find_urls("https://example.com/path?q=1"), vec![(0, 28)]);
        assert_eq!(find_urls("ftp://host/x"), vec![(0, 12)]);
        // A bare domain needs a 2+ letter suffix.
        assert!(find_urls("www.example.c").is_empty());
        assert!(find_urls("www.").is_empty());
        assert!(find_urls("no urls here").is_empty());
        // Two URLs in one string, non-overlapping.
        assert_eq!(
            find_urls("a http://x.y b www.a.io"),
            vec![(2, 12), (15, 23)]
        );
        // The excluded set ends a match.
        assert_eq!(find_urls("<https://x.y>"), vec![(1, 12)]);
    }

    #[test]
    fn phone_scanner_is_non_overlapping() {
        assert_eq!(find_phone_separators("555-1234"), vec![2]);
        assert_eq!(find_phone_separators("555-123-4567"), vec![2, 6]);
        assert_eq!(find_phone_separators("123.456"), vec![2]);
        assert_eq!(find_phone_separators("a-b"), Vec::<usize>::new());
        assert_eq!(find_phone_separators("1\u{2013}2"), vec![0]);
    }

    #[test]
    fn khmer_subscript_suppressed() {
        // A segment ending in U+17D2 must not yield its start.
        let text = "ក\u{17d2}ខ";
        // Segment 0 is "ក\u{17d2}", which ends with the subscript sign.
        let got = apply_break_rules(text, &[0, 6]);
        assert_eq!(got, vec![6]);
    }
}
