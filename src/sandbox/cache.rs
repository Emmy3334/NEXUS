//! Resolve the on-disk Wasm module cache (`NEXUS_WASM_CACHE` / `~/.nexus/wasm`).

use std::env;
use std::path::{Path, PathBuf};

/// Directory that stores `name.wasm` modules for heal / `sandbox`.
#[must_use]
pub fn cache_dir() -> PathBuf {
    if let Ok(explicit) = env::var("NEXUS_WASM_CACHE") {
        if !explicit.is_empty() {
            return PathBuf::from(explicit);
        }
    }
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".nexus")
        .join("wasm")
}

/// `cache_dir()/{stem}.wasm` when that file exists.
#[must_use]
pub fn resolve_named(name: &str) -> Option<PathBuf> {
    let path = cached_wasm_path(name)?;
    path.is_file().then_some(path)
}

/// Path `cache_dir()/{stem}.wasm` (does not require the file to exist).
#[must_use]
pub fn cached_wasm_path(name: &str) -> Option<PathBuf> {
    Some(cache_dir().join(format!("{}.wasm", module_stem(name)?)))
}

/// Bare module name without a trailing `.wasm`.
#[must_use]
pub(crate) fn module_stem(name: &str) -> Option<&str> {
    let base = Path::new(name).file_name()?.to_str()?;
    Some(base.strip_suffix(".wasm").unwrap_or(base))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}
