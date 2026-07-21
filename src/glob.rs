//! Pathname expansion (`*`, `?`, `[…]`) after quote / `$` expansion.
//!
//! Only characters flagged active in [`ExpandedWord`] participate as metas.
//! No match → the literal word is kept (bash / tcsh-`nonomatch` style).

use crate::expand::ExpandedWord;

use std::fs;
use std::path::{Path, PathBuf};

/// Expand active globs in `word` into one or more pathnames (sorted).
///
/// If there is no active glob, or no filesystem match, returns a single
/// entry equal to the expanded text.
#[must_use]
pub fn expand_globs(word: &ExpandedWord) -> Vec<String> {
    if !word.has_active_glob() {
        return vec![word.as_str().to_owned()];
    }

    let pattern: Vec<(char, bool)> = word
        .as_str()
        .chars()
        .zip(word.glob_meta().iter().copied())
        .collect();

    let mut matches = match_pattern(&pattern);
    if matches.is_empty() {
        return vec![word.as_str().to_owned()];
    }
    matches.sort();
    matches
}

/// Expand a redirect target: exactly one pathname, or [`AmbiguousGlob`].
pub fn expand_globs_one(word: &ExpandedWord) -> Result<String, AmbiguousGlob> {
    let matches = expand_globs(word);
    match matches.as_slice() {
        [one] => Ok(one.clone()),
        _ => Err(AmbiguousGlob),
    }
}

/// More than one pathname matched a redirect glob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmbiguousGlob;

fn match_pattern(pattern: &[(char, bool)]) -> Vec<String> {
    if pattern.is_empty() {
        return Vec::new();
    }

    let absolute = pattern.first().is_some_and(|(c, meta)| *c == '/' && !meta);
    let parts = split_pattern_components(pattern);
    if parts.is_empty() {
        return if absolute {
            vec!["/".to_owned()]
        } else {
            Vec::new()
        };
    }

    let mut bases: Vec<PathBuf> = if absolute {
        vec![PathBuf::from("/")]
    } else {
        vec![PathBuf::from(".")]
    };

    for part in &parts {
        let mut next = Vec::new();
        for base in &bases {
            next.extend(expand_component(base, part));
        }
        bases = next;
        if bases.is_empty() {
            return Vec::new();
        }
    }

    bases
        .into_iter()
        .map(|p| path_display(&p, absolute))
        .collect()
}

fn path_display(path: &Path, absolute: bool) -> String {
    if absolute {
        return path.to_string_lossy().into_owned();
    }
    let stripped = path.strip_prefix(".").unwrap_or(path);
    if stripped.as_os_str().is_empty() {
        ".".to_owned()
    } else {
        stripped.to_string_lossy().into_owned()
    }
}

fn split_pattern_components(pattern: &[(char, bool)]) -> Vec<Vec<(char, bool)>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    for &(c, meta) in pattern {
        if c == '/' && !meta {
            if !current.is_empty() {
                parts.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push((c, meta));
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn expand_component(base: &Path, pattern: &[(char, bool)]) -> Vec<PathBuf> {
    if !component_has_glob(pattern) {
        let name: String = pattern.iter().map(|(c, _)| *c).collect();
        let path = base.join(name);
        if path.exists() {
            vec![path]
        } else {
            Vec::new()
        }
    } else {
        let Ok(read_dir) = fs::read_dir(base) else {
            return Vec::new();
        };

        let mut out = Vec::new();
        for entry in read_dir.flatten() {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            if !dotglob_allows(pattern, name) {
                continue;
            }
            if matches_component(pattern, name) {
                out.push(base.join(name));
            }
        }
        out
    }
}

fn component_has_glob(pattern: &[(char, bool)]) -> bool {
    pattern
        .iter()
        .any(|&(c, meta)| meta && matches!(c, '*' | '?' | '['))
}

fn dotglob_allows(pattern: &[(char, bool)], name: &str) -> bool {
    if !name.starts_with('.') {
        return true;
    }
    pattern.first().is_some_and(|(c, _)| *c == '.')
}

fn matches_component(pattern: &[(char, bool)], name: &str) -> bool {
    let name_chars: Vec<char> = name.chars().collect();
    matches_chars(pattern, 0, &name_chars, 0)
}

fn matches_chars(pattern: &[(char, bool)], mut pi: usize, name: &[char], mut ni: usize) -> bool {
    while pi < pattern.len() {
        let (pc, meta) = pattern[pi];
        if meta && pc == '*' {
            pi += 1;
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
            return false;
        }
        if ni >= name.len() {
            return false;
        }
        if meta && pc == '?' {
            pi += 1;
            ni += 1;
            continue;
        }
        if meta && pc == '[' {
            let Some((next_pi, ok)) = match_bracket(pattern, pi, name[ni]) else {
                return false;
            };
            if !ok {
                return false;
            }
            pi = next_pi;
            ni += 1;
            continue;
        }
        if pc != name[ni] {
            return false;
        }
        pi += 1;
        ni += 1;
    }
    ni == name.len()
}

/// Parse `[…]` at `pi`. Returns `(index_after_class, character_matched)`.
fn match_bracket(pattern: &[(char, bool)], pi: usize, ch: char) -> Option<(usize, bool)> {
    let mut i = pi + 1;
    if i >= pattern.len() {
        return None;
    }
    let mut negated = false;
    if matches!(pattern[i].0, '!' | '^') {
        negated = true;
        i += 1;
    }
    let mut matched = false;
    let mut first = true;
    while i < pattern.len() {
        let c = pattern[i].0;
        if c == ']' && !first {
            let ok = if negated { !matched } else { matched };
            return Some((i + 1, ok));
        }
        first = false;
        if i + 2 < pattern.len() && pattern[i + 1].0 == '-' && pattern[i + 2].0 != ']' {
            let start = c;
            let end = pattern[i + 2].0;
            if start <= ch && ch <= end {
                matched = true;
            }
            i += 3;
            continue;
        }
        if c == ch {
            matched = true;
        }
        i += 1;
    }
    None
}
