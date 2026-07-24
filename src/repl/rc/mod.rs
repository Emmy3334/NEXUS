//! Layered startup files (`.nexusenv`, `.nexusrc`, `.nexuslogin`, `.nexuslogout`).

mod path;

use super::script::source_file;
use super::LoopEnd;
use crate::env::ShellEnvironment;

use std::io::{self, Write};
use std::path::Path;

/// Outcome of resolving and running a startup file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RcLoad {
    /// No path resolved, or the file is missing.
    Skipped,
    /// File finished without `exit`; status seeds the REPL `$?`.
    Continue(u8),
    /// Startup file called `exit`.
    Exit(u8),
}

/// Resolve and run the startup RC when the file exists (RC layer only).
pub fn load_startup_rc<O: Write, E: Write>(
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<RcLoad> {
    let Some(rc_path) = path::resolve_rc_path(shell_env) else {
        return Ok(RcLoad::Skipped);
    };
    source_rc(&rc_path, shell_env, stdout, stderr)
}

/// Load startup files: env → (rc + login when `tty`).
pub fn load_startup_chain<O: Write, E: Write>(
    tty: bool,
    login: bool,
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<RcLoad> {
    if path::norcs_disabled() {
        return Ok(RcLoad::Skipped);
    }
    let status = source_optional(path::resolve_env_path(shell_env), shell_env, stdout, stderr)?;
    if matches!(status, RcLoad::Exit(_)) {
        return Ok(status);
    }
    if !tty {
        return Ok(status);
    }
    let status = fold_load(
        status,
        path::resolve_rc_path(shell_env),
        shell_env,
        stdout,
        stderr,
    )?;
    if matches!(status, RcLoad::Exit(_)) {
        return Ok(status);
    }
    if !login {
        return Ok(status);
    }
    fold_load(
        status,
        path::resolve_login_path(shell_env),
        shell_env,
        stdout,
        stderr,
    )
}

/// Load `.nexusenv` only (script invocations).
pub fn load_startup_env<O: Write, E: Write>(
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<RcLoad> {
    if path::norcs_disabled() {
        return Ok(RcLoad::Skipped);
    }
    source_optional(path::resolve_env_path(shell_env), shell_env, stdout, stderr)
}

/// Source `.nexuslogout` quietly on login-shell exit (no `exit` propagation).
pub fn load_logout<O: Write, E: Write>(
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<()> {
    if path::norcs_disabled() {
        return Ok(());
    }
    let Some(logout) = path::resolve_logout_path(shell_env) else {
        return Ok(());
    };
    let _ = source_rc(&logout, shell_env, stdout, stderr)?;
    Ok(())
}

/// Run `path` like `source` if it is a regular file; missing path is a quiet no-op.
pub fn source_rc<O: Write, E: Write>(
    path: &Path,
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<RcLoad> {
    if !path.is_file() {
        return Ok(RcLoad::Skipped);
    }
    match source_file(path, stdout, stderr, shell_env)? {
        LoopEnd::Exit(code) => Ok(RcLoad::Exit(code)),
        LoopEnd::Status(code) => Ok(RcLoad::Continue(code)),
    }
}

fn source_optional<O: Write, E: Write>(
    path: Option<std::path::PathBuf>,
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<RcLoad> {
    let Some(path) = path else {
        return Ok(RcLoad::Skipped);
    };
    source_rc(&path, shell_env, stdout, stderr)
}

fn fold_load<O: Write, E: Write>(
    prior: RcLoad,
    path: Option<std::path::PathBuf>,
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<RcLoad> {
    let load = source_optional(path, shell_env, stdout, stderr)?;
    Ok(match load {
        RcLoad::Exit(code) => RcLoad::Exit(code),
        RcLoad::Continue(code) => RcLoad::Continue(code),
        RcLoad::Skipped => prior,
    })
}
