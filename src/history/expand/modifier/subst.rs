//! `:s/l/r/` and `:&` substitution helpers.

use super::super::cursor::{bump, peek};
use crate::history::History;

/// Parse `:s<delim>l<delim>r[<delim>]` starting after `s`.
pub(super) fn take_subst(
    s: &str,
    i: &mut usize,
    history: &mut History,
) -> Option<(String, String)> {
    let delim = peek(s, *i)?;
    bump(s, i);
    let left = take_field(s, i, delim)?;
    let right = take_field(s, i, delim).unwrap_or_default();
    let left = if left.is_empty() {
        history.last_subst.as_ref()?.0.clone()
    } else {
        left
    };
    history.last_subst = Some((left.clone(), right.clone()));
    Some((left, right))
}

fn take_field(s: &str, i: &mut usize, delim: char) -> Option<String> {
    let mut out = String::new();
    while let Some(c) = peek(s, *i) {
        if c == delim {
            bump(s, i);
            return Some(out);
        }
        if c == '\\' {
            bump(s, i);
            if let Some(escaped) = bump(s, i) {
                out.push(escaped);
            }
            continue;
        }
        out.push(c);
        bump(s, i);
    }
    Some(out)
}

pub(super) fn substitute(word: &str, left: &str, right: &str, all: bool) -> String {
    if left.is_empty() {
        return word.to_string();
    }
    let right = right.replace('&', left);
    if all {
        word.replace(left, &right)
    } else {
        word.replacen(left, &right, 1)
    }
}

pub(super) fn quote_word(word: &str) -> String {
    let mut out = String::from("'");
    for c in word.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

/// Apply `:s` / `:&` to the first matching word (or all with `global`).
pub(super) fn map_subst(
    words: &mut [String],
    global: bool,
    repeat: bool,
    left: &str,
    right: &str,
) -> Result<(), super::super::HistoryError> {
    use super::super::HistoryError;
    if words.is_empty() || left.is_empty() {
        return Err(HistoryError::BadModifier);
    }
    let mut any = false;
    if global {
        for w in words.iter_mut() {
            if w.contains(left) {
                *w = apply_subst_word(w, left, right, repeat);
                any = true;
            }
        }
    } else if let Some(idx) = words.iter().position(|w| w.contains(left)) {
        words[idx] = apply_subst_word(&words[idx], left, right, repeat);
        any = true;
    }
    if any {
        Ok(())
    } else {
        Err(HistoryError::BadModifier)
    }
}

fn apply_subst_word(word: &str, left: &str, right: &str, repeat: bool) -> String {
    let mut w = word.to_string();
    loop {
        let next = substitute(&w, left, right, false);
        if next == w {
            break;
        }
        w = next;
        if !repeat {
            break;
        }
    }
    w
}
