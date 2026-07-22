//! Install a Wasm module into the cache from WebAssembly Text.

use super::cache::cache_dir;

use std::fs;
use std::io;
use std::path::Path;

/// Assemble `wat` and write `cache_dir()/name.wasm` (creates the cache dir).
pub fn install_from_wat(name: &str, wat: &str) -> io::Result<std::path::PathBuf> {
    install_from_wat_into(&cache_dir(), name, wat)
}

/// Like [`install_from_wat`], but into an explicit directory (tests).
pub fn install_from_wat_into(dir: &Path, name: &str, wat: &str) -> io::Result<std::path::PathBuf> {
    let bytes = wat::parse_str(wat)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, format!("wat: {err}")))?;
    fs::create_dir_all(dir)?;
    let path = dir.join(format!("{name}.wasm"));
    fs::write(&path, bytes)?;
    Ok(path)
}
