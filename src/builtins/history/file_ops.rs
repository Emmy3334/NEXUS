//! Histfile and clear operations for `history`.

use crate::env::ShellEnvironment;

use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub(super) fn clear_history(shell_env: &mut ShellEnvironment) {
    shell_env.history.clear();
}

pub(super) fn save_history(
    shell_env: &ShellEnvironment,
    path: Option<&Path>,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let resolved = resolve_histfile(shell_env, path);
    match shell_env.history.save_to(&resolved) {
        Ok(()) => Ok(0),
        Err(err) => {
            writeln!(stderr, "history: {}: {err}.", resolved.display())?;
            Ok(1)
        }
    }
}

pub(super) fn load_history(
    shell_env: &mut ShellEnvironment,
    path: Option<&Path>,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let resolved = resolve_histfile(shell_env, path);
    match shell_env.history.load_append(&resolved) {
        Ok(()) => Ok(0),
        Err(err) => {
            writeln!(stderr, "history: {}: {err}.", resolved.display())?;
            Ok(1)
        }
    }
}

pub(super) fn merge_history(
    shell_env: &mut ShellEnvironment,
    path: Option<&Path>,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let resolved = resolve_histfile(shell_env, path);
    match shell_env.history.merge_from(&resolved) {
        Ok(()) => Ok(0),
        Err(err) => {
            writeln!(stderr, "history: {}: {err}.", resolved.display())?;
            Ok(1)
        }
    }
}

fn resolve_histfile(shell_env: &ShellEnvironment, path: Option<&Path>) -> PathBuf {
    if let Some(p) = path {
        return p.to_path_buf();
    }
    if let Some(hf) = shell_env.get_local("histfile") {
        return PathBuf::from(hf);
    }
    let home = shell_env
        .lookup("home")
        .or_else(|| shell_env.lookup("HOME"))
        .unwrap_or(".");
    PathBuf::from(home).join(".nexus_history")
}
