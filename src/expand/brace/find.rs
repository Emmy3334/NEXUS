//! Locate the first expandable `{…,…}` span (quote/escape aware).

use super::parse;

/// `(prefix, alternatives, suffix)` for the first brace group, if any.
pub(super) fn first_brace(raw: &str) -> Option<(String, Vec<String>, String)> {
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                i = i.saturating_add(2);
            }
            '\'' => {
                i = skip_until(&chars, i + 1, '\'');
            }
            '"' => {
                i = skip_double(&chars, i + 1);
            }
            '{' => {
                if let Some((alts, end)) = parse::alternatives(&chars, i) {
                    let prefix: String = chars[..i].iter().collect();
                    let suffix: String = chars[end..].iter().collect();
                    return Some((prefix, alts, suffix));
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

fn skip_until(chars: &[char], mut i: usize, end: char) -> usize {
    while i < chars.len() {
        if chars[i] == end {
            return i + 1;
        }
        i += 1;
    }
    i
}

fn skip_double(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() {
        match chars[i] {
            '\\' => i = i.saturating_add(2),
            '"' => return i + 1,
            _ => i += 1,
        }
    }
    i
}
