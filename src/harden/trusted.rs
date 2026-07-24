//! Opt-in absolute-path allowlist for host externals (`trusted_bin` / `NEXUS_TRUSTED_BIN`).

use crate::env::ShellEnvironment;
use crate::harden::config;

use std::path::{Component, Path, PathBuf};

/// Non-empty allowlist when the feature is on; `None` means unrestricted.
#[must_use]
pub fn allowlist(env: &ShellEnvironment) -> Option<Vec<PathBuf>> {
    let raw = spec(env)?;
    let entries: Vec<PathBuf> = raw
        .split(':')
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.starts_with('/'))
        .map(PathBuf::from)
        .collect();
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// `Ok(())` if unrestricted or `resolved` matches the allowlist.
pub fn check(argv0: &str, resolved: &str, env: &ShellEnvironment) -> Result<(), String> {
    let Some(allow) = allowlist(env) else {
        return Ok(());
    };
    // Unresolved bare names fall through to normal NotFound / heal.
    if !resolved.contains('/') {
        return Ok(());
    }
    if is_allowed(Path::new(resolved), &allow) {
        Ok(())
    } else {
        Err(format!("{argv0}: not in trusted_bin allowlist."))
    }
}

fn spec(env: &ShellEnvironment) -> Option<String> {
    if let Some(v) = env.lookup("trusted_bin") {
        return if config::is_off(v) || v.trim().is_empty() {
            None
        } else {
            Some(v.to_owned())
        };
    }
    match std::env::var("NEXUS_TRUSTED_BIN") {
        Ok(v) if config::is_off(&v) || v.trim().is_empty() => None,
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

fn is_allowed(program: &Path, allow: &[PathBuf]) -> bool {
    let candidate = normalize(program);
    allow.iter().any(|entry| {
        let as_prefix = entry.to_str().is_some_and(|s| s.ends_with('/')) || entry.is_dir();
        let entry = normalize(entry);
        if as_prefix {
            candidate.starts_with(&entry)
        } else {
            candidate == entry
        }
    })
}

fn normalize(path: &Path) -> PathBuf {
    if let Ok(canon) = path.canonicalize() {
        return canon;
    }
    lexical_normalize(path)
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}
