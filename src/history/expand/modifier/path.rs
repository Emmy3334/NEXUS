//! Path-style and case modifiers.

pub(super) fn head(word: &str) -> String {
    match word.rsplit_once('/') {
        Some((h, _)) if !h.is_empty() => h.to_string(),
        Some((_, _)) => "/".to_string(),
        None => word.to_string(),
    }
}

pub(super) fn tail(word: &str) -> String {
    word.rsplit('/').next().unwrap_or(word).to_string()
}

pub(super) fn root(word: &str) -> String {
    match word.rsplit_once('.') {
        Some((r, _)) if !r.is_empty() && r != word.trim_start_matches('.') => r.to_string(),
        _ => word.to_string(),
    }
}

pub(super) fn ext(word: &str) -> String {
    match word.rsplit_once('.') {
        Some((r, e)) if !r.is_empty() && !e.is_empty() && r != word.trim_start_matches('.') => {
            e.to_string()
        }
        _ => String::new(),
    }
}

pub(super) fn upper_first(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut done = false;
    for c in word.chars() {
        if !done && c.is_lowercase() {
            out.extend(c.to_uppercase());
            done = true;
        } else {
            out.push(c);
        }
    }
    out
}

pub(super) fn lower_first(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut done = false;
    for c in word.chars() {
        if !done && c.is_uppercase() {
            out.extend(c.to_lowercase());
            done = true;
        } else {
            out.push(c);
        }
    }
    out
}
