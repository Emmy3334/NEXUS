//! Character-class helpers for history designators.

use super::cursor::{bump, peek};

pub(super) fn read_number(s: &str, i: &mut usize) -> Option<usize> {
    let ch = peek(s, *i)?;
    if !ch.is_ascii_digit() {
        return None;
    }
    let mut n: usize = 0;
    while let Some(c) = peek(s, *i) {
        if !c.is_ascii_digit() {
            break;
        }
        n = n
            .saturating_mul(10)
            .saturating_add(c.to_digit(10).unwrap_or(0) as usize);
        bump(s, i);
    }
    Some(n)
}

pub(super) fn is_prefix_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '.' || c == '/'
}

pub(super) fn is_prefix_continue(c: char) -> bool {
    is_prefix_start(c) || c.is_ascii_digit() || c == '-'
}

pub(super) fn is_literal_bang_follower(c: char) -> bool {
    c.is_whitespace() || c == '=' || c == '('
}

pub(super) fn split_words(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
}

pub(super) fn join_words(words: &[String]) -> String {
    words.join(" ")
}
