//! Home substitution and unique-prefix path shortening.

use super::prefix::{enforce_max, unique_prefix};
use super::tilde::with_tilde;
use std::path::{Component, Path, PathBuf};

const MAX_LEN: usize = 40;

/// `~/Doc/G/NEXUS`-style path for the prompt.
#[must_use]
pub fn display_path(cwd: &str, home: &str) -> String {
    let abs = PathBuf::from(cwd);
    let names = normal_names(&abs);
    if names.is_empty() {
        return "/".into();
    }
    let shortened = shorten_names(&names, &abs);
    with_tilde(&shortened, home, &abs)
}

pub(super) fn normal_names(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
}

fn shorten_names(names: &[String], abs: &Path) -> Vec<String> {
    let mut out = names.to_vec();
    let last = names.len().saturating_sub(1);
    for i in 0..last {
        let parent = ancestor(abs, names.len() - i);
        out[i] = unique_prefix(&names[i], parent.as_deref());
    }
    enforce_max(&mut out, last, MAX_LEN);
    out
}

fn ancestor(abs: &Path, steps_up: usize) -> Option<PathBuf> {
    let mut p = abs.to_path_buf();
    for _ in 0..steps_up {
        p = p.parent()?.to_path_buf();
    }
    Some(p)
}
