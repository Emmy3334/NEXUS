//! Resolve layered startup paths under `$NEXUS_DOTDIR` or `$HOME`.

use crate::env::ShellEnvironment;

use std::env;
use std::path::PathBuf;

/// When `NEXUS_NORCS=1`, all startup files are skipped.
#[must_use]
pub(super) fn norcs_disabled() -> bool {
    env::var("NEXUS_NORCS").as_deref() == Ok("1")
}

/// `$NEXUS_DOTDIR` when set and non-empty; else `$home` / `$HOME`.
#[must_use]
pub(super) fn resolve_dotdir(shell_env: &ShellEnvironment) -> Option<PathBuf> {
    if let Ok(dotdir) = env::var("NEXUS_DOTDIR") {
        if !dotdir.is_empty() {
            return Some(PathBuf::from(dotdir));
        }
    }
    shell_env
        .lookup("home")
        .or_else(|| shell_env.lookup("HOME"))
        .map(str::to_owned)
        .or_else(|| env::var("HOME").ok())
        .map(PathBuf::from)
}

/// `$dotdir/.nexusenv` — loaded on every invocation.
#[must_use]
pub(super) fn resolve_env_path(shell_env: &ShellEnvironment) -> Option<PathBuf> {
    resolve_dotdir(shell_env).map(|d| d.join(".nexusenv"))
}

/// `NEXUSRC` if set (empty disables); else `$dotdir/.nexusrc`.
#[must_use]
pub(super) fn resolve_rc_path(shell_env: &ShellEnvironment) -> Option<PathBuf> {
    if let Ok(explicit) = env::var("NEXUSRC") {
        if explicit.is_empty() {
            return None;
        }
        return Some(PathBuf::from(explicit));
    }
    resolve_dotdir(shell_env).map(|d| d.join(".nexusrc"))
}

/// `$dotdir/.nexuslogin` — login shells only.
#[must_use]
pub(super) fn resolve_login_path(shell_env: &ShellEnvironment) -> Option<PathBuf> {
    resolve_dotdir(shell_env).map(|d| d.join(".nexuslogin"))
}

/// `$dotdir/.nexuslogout` — sourced on exit of login shells.
#[must_use]
pub(super) fn resolve_logout_path(shell_env: &ShellEnvironment) -> Option<PathBuf> {
    resolve_dotdir(shell_env).map(|d| d.join(".nexuslogout"))
}
