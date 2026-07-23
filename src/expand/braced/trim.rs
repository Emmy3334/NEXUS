//! Prefix/suffix pattern removal for `${var#pat}` / `${var%pat}` (and `##` / `%%`).

/// Remove a matching prefix; `longest` selects `##` vs `#`.
#[must_use]
pub(super) fn prefix(value: &str, pat: &str, longest: bool) -> String {
    let chars: Vec<char> = value.chars().collect();
    if longest {
        for end in (0..=chars.len()).rev() {
            if affix_match(pat, &chars[..end]) {
                return chars[end..].iter().collect();
            }
        }
    } else {
        for end in 0..=chars.len() {
            if affix_match(pat, &chars[..end]) {
                return chars[end..].iter().collect();
            }
        }
    }
    value.to_owned()
}

/// Remove a matching suffix; `longest` selects `%%` vs `%`.
#[must_use]
pub(super) fn suffix(value: &str, pat: &str, longest: bool) -> String {
    let chars: Vec<char> = value.chars().collect();
    if longest {
        for start in 0..=chars.len() {
            if affix_match(pat, &chars[start..]) {
                return chars[..start].iter().collect();
            }
        }
    } else {
        for start in (0..=chars.len()).rev() {
            if affix_match(pat, &chars[start..]) {
                return chars[..start].iter().collect();
            }
        }
    }
    value.to_owned()
}

fn affix_match(pat: &str, text: &[char]) -> bool {
    match_chars(&pat.chars().collect::<Vec<_>>(), text)
}

fn match_chars(pat: &[char], text: &[char]) -> bool {
    match (pat.first(), text.first()) {
        (None, None) => true,
        (Some('*'), _) => match_star(&pat[1..], text),
        (Some('?'), Some(_)) => match_chars(&pat[1..], &text[1..]),
        (Some(p), Some(t)) if p == t => match_chars(&pat[1..], &text[1..]),
        _ => false,
    }
}

fn match_star(pat: &[char], text: &[char]) -> bool {
    if match_chars(pat, text) {
        return true;
    }
    !text.is_empty() && match_star(pat, &text[1..])
}
