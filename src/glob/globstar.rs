//! Recursive `**` path-component expansion (globstar).

use std::fs;
use std::path::{Path, PathBuf};

/// A component that is exactly active `**`.
#[must_use]
pub(super) fn is_globstar(pattern: &[(char, bool)]) -> bool {
    matches!(pattern, [('*', true), ('*', true)])
}

/// Expand `**` under `base`.
///
/// When `dirs_only` (more pattern parts follow), include `base` and descendant
/// directories. When trailing, include all descendant files and directories.
pub(super) fn expand(base: &Path, dirs_only: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if dirs_only {
        out.push(base.to_path_buf());
    }
    walk(base, dirs_only, &mut out);
    out
}

fn walk(dir: &Path, dirs_only: bool, out: &mut Vec<PathBuf>) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let is_dir = path.is_dir();
        if dirs_only {
            if is_dir {
                out.push(path.clone());
                walk(&path, true, out);
            }
        } else {
            out.push(path.clone());
            if is_dir {
                walk(&path, false, out);
            }
        }
    }
}
