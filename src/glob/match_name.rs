//! Match a single path component against a glob pattern.

use super::bracket::match_bracket;

/// Whether the component pattern contains an active glob metacharacter.
pub(super) fn component_has_glob(pattern: &[(char, bool)]) -> bool {
    pattern
        .iter()
        .any(|&(c, meta)| meta && matches!(c, '*' | '?' | '['))
}

pub(super) fn matches_component(pattern: &[(char, bool)], name: &str) -> bool {
    let name_chars: Vec<char> = name.chars().collect();
    matches_chars(pattern, 0, &name_chars, 0)
}

fn matches_chars(pattern: &[(char, bool)], mut pi: usize, name: &[char], mut ni: usize) -> bool {
    while pi < pattern.len() {
        let (pc, meta) = pattern[pi];
        if meta && pc == '*' {
            return match_star(pattern, pi + 1, name, ni);
        }
        if ni >= name.len() {
            return false;
        }
        if !match_one(pattern, &mut pi, name, &mut ni) {
            return false;
        }
    }
    ni == name.len()
}

fn match_star(pattern: &[(char, bool)], pi: usize, name: &[char], mut ni: usize) -> bool {
    if pi == pattern.len() {
        return true;
    }
    while ni <= name.len() {
        if matches_chars(pattern, pi, name, ni) {
            return true;
        }
        if ni == name.len() {
            break;
        }
        ni += 1;
    }
    false
}

fn match_one(pattern: &[(char, bool)], pi: &mut usize, name: &[char], ni: &mut usize) -> bool {
    let (pc, meta) = pattern[*pi];
    if meta && pc == '?' {
        *pi += 1;
        *ni += 1;
        return true;
    }
    if meta && pc == '[' {
        let Some((next_pi, ok)) = match_bracket(pattern, *pi, name[*ni]) else {
            return false;
        };
        if !ok {
            return false;
        }
        *pi = next_pi;
        *ni += 1;
        return true;
    }
    if pc != name[*ni] {
        return false;
    }
    *pi += 1;
    *ni += 1;
    true
}
