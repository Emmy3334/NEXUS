//! PATH jail / rlimit enable flags (`path_jail` / `NEXUS_PATH_JAIL`, `rlimit` / `NEXUS_RLIMIT`).
//! `trusted_bin` / `NEXUS_TRUSTED_BIN` is handled in [`super::trusted`].

use crate::env::ShellEnvironment;
use crate::pathfind;

/// Absolute-only PATH filtering (default **on**).
#[must_use]
pub fn path_jail_enabled(env: &ShellEnvironment) -> bool {
    if let Some(v) = env.lookup("path_jail") {
        return !is_off(v);
    }
    match std::env::var("NEXUS_PATH_JAIL") {
        Ok(v) => !is_off(&v),
        Err(_) => true,
    }
}

/// Opt-in child `setrlimit` (default **off**).
#[must_use]
pub fn rlimit_enabled(env: &ShellEnvironment) -> bool {
    if let Some(v) = env.lookup("rlimit") {
        return is_on(v);
    }
    match std::env::var("NEXUS_RLIMIT") {
        Ok(v) => is_on(&v),
        Err(_) => false,
    }
}

/// `PATH` value for search/spawn: sanitized when jail is on.
#[must_use]
pub fn effective_path(env: &ShellEnvironment) -> String {
    let raw = env.lookup("PATH").unwrap_or("");
    if path_jail_enabled(env) {
        pathfind::sanitize_path(raw)
    } else {
        raw.to_owned()
    }
}

pub(super) fn is_off(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "no" | "off"
    )
}

pub(super) fn is_on(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
