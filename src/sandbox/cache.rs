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

/// `cache_dir()/name.wasm` when that file exists.
#[must_use]
pub fn resolve_named(name: &str) -> Option<PathBuf> {
    let base = Path::new(name).file_name()?.to_str()?;
    let path = cache_dir().join(format!("{base}.wasm"));
    path.is_file().then_some(path)
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}
