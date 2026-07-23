//! List / remove / install Wasm modules in the cache.

use super::cache::{cache_dir, cached_wasm_path, module_stem};
use super::wat_install::install_from_wat_into;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Sorted bare names of `*.wasm` files in the cache (empty if missing).
#[must_use]
pub fn list_names() -> Vec<String> {
    let Ok(entries) = fs::read_dir(cache_dir()) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("wasm") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            names.push(stem.to_owned());
        }
    }
    names.sort();
    names
}

/// Delete `name.wasm` from the cache.
pub fn remove_named(name: &str) -> io::Result<()> {
    let path = cached_wasm_path(name).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("invalid name: {name}"))
    })?;
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{name}: Wasm module not found."),
        ));
    }
    fs::remove_file(path)
}

/// Install a local `.wasm` or `.wat` file into the cache as `{name}.wasm`.
pub fn install_from_path(path: &Path, name: Option<&str>) -> io::Result<PathBuf> {
    let stem = name
        .and_then(module_stem)
        .or_else(|| default_stem(path))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing module name"))?;
    let dir = cache_dir();
    if looks_like_wat(path) {
        let wat = fs::read_to_string(path)?;
        return install_from_wat_into(&dir, stem, &wat);
    }
    let bytes = fs::read(path)?;
    if !bytes.starts_with(b"\0asm") && path.extension().is_none_or(|e| e != "wasm") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: not a Wasm module", path.display()),
        ));
    }
    fs::create_dir_all(&dir)?;
    let dest = dir.join(format!("{stem}.wasm"));
    fs::write(&dest, bytes)?;
    Ok(dest)
}

fn default_stem(path: &Path) -> Option<&str> {
    let stem = path.file_stem()?.to_str()?;
    module_stem(stem)
}

fn looks_like_wat(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("wat")
}
