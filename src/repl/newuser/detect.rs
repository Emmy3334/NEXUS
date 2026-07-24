//! When to offer the first-run wizard (mirrors zsh/newuser gates).

use crate::env::ShellEnvironment;
use crate::repl::rc::path;

use std::env;

/// True when an interactive first-run install should be offered.
#[must_use]
pub fn should_offer(shell_env: &ShellEnvironment) -> bool {
    if path::norcs_disabled() {
        return false;
    }
    if env::var("NEXUS_NONEWUSER").as_deref() == Ok("1") {
        return false;
    }
    if env::var_os("NEXUSRC").is_some() {
        return false;
    }
    if is_root() {
        return false;
    }
    let Some(dotdir) = path::resolve_dotdir(shell_env) else {
        return false;
    };
    if !dotdir_writable(&dotdir) {
        return false;
    }
    if path::any_dotfile_present(shell_env) && env::var("NEXUS_NEWUSER").as_deref() != Ok("1") {
        return false;
    }
    terminal_large_enough()
}

fn is_root() -> bool {
    #[cfg(unix)]
    {
        nix::unistd::Uid::effective().is_root()
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn dotdir_writable(dotdir: &std::path::Path) -> bool {
    if !dotdir.is_dir() {
        return false;
    }
    let probe = dotdir.join(".nexus-newuser-write-probe");
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

fn terminal_large_enough() -> bool {
    let lines = env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24_u32);
    let cols = env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80_u32);
    lines >= 15 && cols >= 72
}
