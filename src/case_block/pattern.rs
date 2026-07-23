//! Glob-style pattern match for `case` arms (`*` / `?` / `|`).

/// True if `subject` matches any `|`-separated alternative in `pattern`.
#[must_use]
pub fn matches_any(subject: &str, pattern: &str) -> bool {
    pattern.split('|').any(|alt| glob_match(alt, subject))
}

fn glob_match(pat: &str, text: &str) -> bool {
    match_chars(
        &pat.chars().collect::<Vec<_>>(),
        &text.chars().collect::<Vec<_>>(),
    )
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
