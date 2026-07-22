//! Startup file (`~/.nexusrc`) loading.

mod path;

use super::script::source_file;
use super::LoopEnd;
use crate::env::ShellEnvironment;

use std::io::{self, Write};
use std::path::Path;

/// Outcome of resolving and running a startup RC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RcLoad {
    /// No path resolved, or the file is missing.
    Skipped,
    /// RC finished without `exit`; `last_status` seeds the REPL `$?`.
    Continue(u8),
    /// RC called `exit`.
    Exit(u8),
}

/// Resolve and run the startup RC when the file exists.
pub fn load_startup_rc<O: Write, E: Write>(
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<RcLoad> {
    let Some(path) = path::resolve_rc_path(shell_env) else {
        return Ok(RcLoad::Skipped);
    };
    source_rc(&path, shell_env, stdout, stderr)
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
