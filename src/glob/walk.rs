//! Top-level glob expand and directory walk.

use super::component::expand_component;
use super::components::split_pattern_components;
use super::globstar;
use super::qualifier::{filter_matches, strip_qualifier};
use crate::expand::ExpandedWord;

use std::path::{Path, PathBuf};

/// More than one pathname matched a redirect glob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmbiguousGlob;

/// Expand active globs in `word` into one or more pathnames (sorted).
///
/// If there is no active glob, or no filesystem match, returns a single
/// entry equal to the expanded text.
#[must_use]
pub fn expand_globs(word: &ExpandedWord) -> Vec<String> {
    if !word.has_active_glob() {
        return vec![word.as_str().to_owned()];
    }
    let (pattern, qual) = strip_qualifier(
        &word
            .as_str()
            .chars()
            .zip(word.glob_meta().iter().copied())
            .collect::<Vec<_>>(),
    );
    let mut matches = match_pattern(&pattern);
    if matches.is_empty() {
        return vec![word.as_str().to_owned()];
    }
    if let Some(qual) = qual {
        matches = filter_matches(matches, qual);
        if matches.is_empty() {
            return vec![word.as_str().to_owned()];
        }
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
    walk_components(parts, absolute)
}

fn walk_components(parts: Vec<Vec<(char, bool)>>, absolute: bool) -> Vec<String> {
    let mut bases: Vec<PathBuf> = if absolute {
        vec![PathBuf::from("/")]
    } else {
        vec![PathBuf::from(".")]
    };
    for (i, part) in parts.iter().enumerate() {
        let mut next = Vec::new();
        let dirs_only = i + 1 < parts.len();
        for base in &bases {
            if globstar::is_globstar(part) {
                next.extend(globstar::expand(base, dirs_only));
            } else {
                next.extend(expand_component(base, part));
            }
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
