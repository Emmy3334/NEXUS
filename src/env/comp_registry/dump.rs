//! Load/save the completion dump file (atomic write via temp + rename).

use super::audit::is_safe;
use super::parse::parse_dump;
use super::{CompRegistry, DUMP_VERSION};
use crate::env::ShellEnvironment;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub use super::audit::is_safe as dump_path_is_safe;

/// Resolve dump path: `$NEXUS_COMPDUMP`, local `compdump`, else `$HOME/.nexus_compdump`.
#[must_use]
pub fn resolve_dump_path(env: &ShellEnvironment) -> Option<PathBuf> {
    if let Some(path) = env.lookup("NEXUS_COMPDUMP") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    if let Some(path) = env.get_local("compdump") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    env.lookup("HOME")
        .or_else(|| env.get_local("home"))
        .map(|home| PathBuf::from(home).join(".nexus_compdump"))
}

/// Quiet TTY boot load into `env.comp_registry` (invalid/missing dump is OK).
pub fn boot_load(env: &mut ShellEnvironment) {
    let Some(path) = resolve_dump_path(env) else {
        return;
    };
    if let Ok(Some(reg)) = load_dump(&path) {
        env.comp_registry = reg;
    }
}

/// Load dump when header count and version match body lines.
pub fn load_dump(path: &Path) -> io::Result<Option<CompRegistry>> {
    if !path.is_file() {
        return Ok(None);
    }
    if !is_safe(path)? {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    parse_dump(&text)
}

/// Write registry to dump path (atomic temp + rename).
pub fn save_dump(path: &Path, registry: &CompRegistry) -> io::Result<()> {
    if path.exists() && !is_safe(path)? {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "compdump: dump path is world-writable",
        ));
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let tmp = path.with_extension("tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        write_dump(registry, &mut file)?;
        file.sync_all()?;
    }
    fs::rename(tmp, path)
}

fn write_dump(registry: &CompRegistry, out: &mut impl Write) -> io::Result<()> {
    let count = registry.words.len();
    writeln!(out, "#files:{count}\tversion:{DUMP_VERSION}")?;
    for (cmd, words) in &registry.words {
        writeln!(out, "{cmd}\t{}", words.join(" "))?;
    }
    Ok(())
}
