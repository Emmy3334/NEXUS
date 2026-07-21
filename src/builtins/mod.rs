//! Shell builtins for Minishell1: `cd`, `setenv`, `unsetenv`, `env`, `exit`.
//!
//! Each command lives in its own module; this file dispatches by `argv[0]`.

mod cd;
mod env;
mod exit;
mod setenv;
mod unsetenv;

use crate::env::ShellEnvironment;

use std::io::{self, Write};

/// Outcome of a recognized builtin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "builtin status vs shell exit must be handled by the caller"]
pub enum BuiltinResult {
    /// Keep the REPL running with this status.
    Status(u8),
    /// Terminate the shell with this status (`exit`).
    Exit(u8),
}

/// Whether `name` is a Minishell builtin.
#[must_use]
pub fn is_builtin(name: &str) -> bool {
    matches!(name, "cd" | "setenv" | "unsetenv" | "env" | "exit")
}

/// Run a builtin if `argv[0]` matches one; otherwise return `Ok(None)`.
pub fn try_run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<Option<BuiltinResult>> {
    let Some(name) = argv.first().map(String::as_str) else {
        return Ok(None);
    };

    let result = match name {
        "cd" => BuiltinResult::Status(cd::cd(argv, shell_env, stderr)?),
        "setenv" => BuiltinResult::Status(setenv::setenv(argv, shell_env, stdout, stderr)?),
        "unsetenv" => BuiltinResult::Status(unsetenv::unsetenv(argv, shell_env, stderr)?),
        "env" => BuiltinResult::Status(env::env_cmd(argv, shell_env, stdout, stderr)?),
        "exit" => exit::exit_cmd(argv, last_status, stderr)?,
        _ => return Ok(None),
    };

    Ok(Some(result))
}
