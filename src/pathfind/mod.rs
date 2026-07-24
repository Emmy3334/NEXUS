//! Locate executables on `PATH` (for `which` / `where` / spawn).

mod hash;
mod jail;

pub use jail::sanitize_path;

use std::path::{Path, PathBuf};

/// Absolute/relative path if executable, else search `path_var` (cached).
#[must_use]
pub fn resolve_first(name: &str, path_var: &str) -> Option<PathBuf> {
    if name.contains('/') {
        return is_executable(Path::new(name));
    }
    if let Some(hit) = hash::lookup(name, path_var) {
        return Some(hit);
    }
    for dir in path_var.split(':').filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(name);
        if let Some(path) = is_executable(&candidate) {
            hash::remember(name, path_var, path.clone());
            return Some(path);
        }
    }
    None
}

/// Every executable match on `PATH` (or the path itself).
#[must_use]
pub fn resolve_all(name: &str, path_var: &str) -> Vec<PathBuf> {
    if name.contains('/') {
        return is_executable(Path::new(name)).into_iter().collect();
    }
    let mut out = Vec::new();
    for dir in path_var.split(':').filter(|d| !d.is_empty()) {
        let candidate = Path::new(dir).join(name);
        if let Some(path) = is_executable(&candidate) {
            out.push(path);
        }
    }
    out
}

/// Hashed command names with the given prefix (after at least one resolve).
#[must_use]
pub fn cached_names(prefix: &str, path_var: &str) -> Vec<String> {
    hash::entries(path_var)
        .into_iter()
        .filter_map(|(name, _)| name.starts_with(prefix).then_some(name))
        .collect()
}

/// PATH command names matching `prefix` (directory listing cache).
#[must_use]
pub fn list_commands(prefix: &str, path_var: &str) -> Vec<String> {
    hash::list_commands(prefix, path_var)
}

/// Clear the command hash (e.g. `hash -r` or PATH change).
pub fn clear_hash() {
    hash::clear();
}

/// Sorted `(name, path)` pairs in the command hash.
#[must_use]
pub fn hash_entries(path_var: &str) -> Vec<(String, PathBuf)> {
    hash::entries(path_var)
}

pub(super) fn is_executable(path: &Path) -> Option<PathBuf> {
    let meta = std::fs::metadata(path).ok()?;
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
