//! Locate executables on `PATH` (for `which` / `where`).

use std::fs;
use std::path::{Path, PathBuf};

/// Absolute/relative path if executable, else search `path_var`.
#[must_use]
pub fn resolve_first(name: &str, path_var: &str) -> Option<PathBuf> {
    if name.contains('/') {
        return executable_path(Path::new(name));
    }
    for dir in path_var.split(':').filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(name);
        if let Some(path) = executable_path(&candidate) {
            return Some(path);
        }
    }
    None
}

/// Every executable match on `PATH` (or the path itself).
#[must_use]
pub fn resolve_all(name: &str, path_var: &str) -> Vec<PathBuf> {
    if name.contains('/') {
        return executable_path(Path::new(name)).into_iter().collect();
    }
    let mut out = Vec::new();
    for dir in path_var.split(':').filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(name);
        if let Some(path) = executable_path(&candidate) {
            out.push(path);
        }
    }
    out
}

fn executable_path(path: &Path) -> Option<PathBuf> {
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o111 == 0 {
            return None;
        }
    }
    Some(path.to_path_buf())
}
