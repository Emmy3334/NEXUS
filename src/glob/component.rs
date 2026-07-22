//! Expand one path component (literal or glob).

use super::match_name::{component_has_glob, matches_component};

use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn expand_component(base: &Path, pattern: &[(char, bool)]) -> Vec<PathBuf> {
    if !component_has_glob(pattern) {
        expand_literal_component(base, pattern)
    } else {
        expand_glob_component(base, pattern)
    }
}

fn expand_literal_component(base: &Path, pattern: &[(char, bool)]) -> Vec<PathBuf> {
    let name: String = pattern.iter().map(|(c, _)| *c).collect();
    let path = base.join(name);
    if path.exists() {
        vec![path]
    } else {
        Vec::new()
    }
}

fn expand_glob_component(base: &Path, pattern: &[(char, bool)]) -> Vec<PathBuf> {
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

fn dotglob_allows(pattern: &[(char, bool)], name: &str) -> bool {
    if !name.starts_with('.') {
        return true;
    }
    pattern.first().is_some_and(|(c, _)| *c == '.')
}
