//! Wasm micro-sandbox: local module cache + Wasmtime WASI runner.

mod cache;
mod engine;
mod exit_code;
mod host;
mod run;
mod wat_install;

pub use cache::{cache_dir, resolve_named};
pub use host::run_host;
pub use run::run_module;
pub use wat_install::{install_from_wat, install_from_wat_into};

use std::io::{self, Write};
use std::path::Path;

/// Run a cached module by command name, or a filesystem `.wasm` path.
pub fn run_named_or_path(
    target: &str,
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    if let Some(path) = resolve_target(target) {
        return run_module(&path, args, stdout, stderr);
    }
    writeln!(stderr, "sandbox: {target}: Wasm module not found.")?;
    Ok(127)
}

/// True when `target` is an existing Wasm file (suffix or magic).
#[must_use]
pub fn is_wasm_path(target: &str) -> bool {
    let path = Path::new(target);
    path.is_file() && looks_like_wasm(target, path)
}

fn resolve_target(target: &str) -> Option<std::path::PathBuf> {
    if is_wasm_path(target) {
        return Some(Path::new(target).to_path_buf());
    }
    resolve_named(target)
}

fn looks_like_wasm(target: &str, path: &Path) -> bool {
    target.ends_with(".wasm") || file_has_wasm_magic(path)
}

fn file_has_wasm_magic(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    bytes.starts_with(b"\0asm")
}
