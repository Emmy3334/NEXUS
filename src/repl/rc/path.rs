//! Resolve the startup RC path (`NEXUSRC` or `~/.nexusrc`).

use crate::env::ShellEnvironment;

use std::env;
use std::path::PathBuf;

/// `NEXUSRC` if set (empty disables); else `$home`/`$HOME` + `/.nexusrc`.
#[must_use]
pub(super) fn resolve_rc_path(shell_env: &ShellEnvironment) -> Option<PathBuf> {
    if let Ok(explicit) = env::var("NEXUSRC") {
        if explicit.is_empty() {
            return None;
        }
        return Some(PathBuf::from(explicit));
    }
    let home = shell_env
        .lookup("home")
        .or_else(|| shell_env.lookup("HOME"))
        .map(str::to_owned)
        .or_else(|| env::var("HOME").ok())?;
    Some(PathBuf::from(home).join(".nexusrc"))
}
