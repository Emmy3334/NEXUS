//! Resolve the default histfile path (`histfile` local or `~/.nexus_history`).

use std::path::PathBuf;

/// Prefer an explicit `histfile` local; else `$home` / `$HOME` / `.nexus_history`.
#[must_use]
pub fn resolve(histfile: Option<&str>, home: Option<&str>) -> PathBuf {
    if let Some(path) = histfile {
        return PathBuf::from(path);
    }
    PathBuf::from(home.unwrap_or(".")).join(".nexus_history")
}
