//! Shell builtins: Minishell1 plus 42sh locals, aliases, history, and jobs.
//!
//! Each command lives in its own module; this file dispatches by `argv[0]`.

mod alias;
mod at;
mod bindkey;
mod cd;
mod env;
mod exit;
mod history;
mod jobs;
mod set;
mod setenv;
mod source;
mod unalias;
mod unset;
mod unsetenv;

use crate::env::ShellEnvironment;

use std::io::{self, Write};

/// Outcome of a recognized builtin.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "builtin status vs shell exit must be handled by the caller"]
pub enum BuiltinResult {
    /// Keep the REPL running with this status.
    Status(u8),
    /// Terminate the shell with this status (`exit`).
    Exit(u8),
    /// Run `path` in the current shell (handled by the REPL).
    Source(String),
}

/// Whether `name` is a shell builtin.
#[must_use]
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "cd" | "setenv"
            | "unsetenv"
            | "env"
            | "exit"
            | "set"
            | "unset"
            | "alias"
            | "unalias"
            | "history"
            | "jobs"
            | "fg"
            | "bg"
            | "source"
            | "."
            | "@"
            | "bindkey"
    )
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
        "cd" => BuiltinResult::Status(cd::cd(argv, shell_env, last_status, stdout, stderr)?),
        "setenv" => BuiltinResult::Status(setenv::setenv(argv, shell_env, stdout, stderr)?),
        "unsetenv" => BuiltinResult::Status(unsetenv::unsetenv(argv, shell_env, stderr)?),
        "env" => BuiltinResult::Status(env::env_cmd(argv, shell_env, stdout, stderr)?),
        "set" => BuiltinResult::Status(set::set(argv, shell_env, stdout, stderr)?),
        "unset" => BuiltinResult::Status(unset::unset(argv, shell_env, stderr)?),
        "alias" => BuiltinResult::Status(alias::alias(argv, shell_env, stdout, stderr)?),
        "unalias" => BuiltinResult::Status(unalias::unalias(argv, shell_env, stderr)?),
        "bindkey" => BuiltinResult::Status(bindkey::bindkey(argv, shell_env, stdout, stderr)?),
        "history" => BuiltinResult::Status(history::history_cmd(argv, shell_env, stdout, stderr)?),
        "jobs" => BuiltinResult::Status(jobs::jobs_cmd(argv, shell_env, stdout, stderr)?),
        "fg" => BuiltinResult::Status(jobs::fg_cmd(argv, shell_env, stderr)?),
        "bg" => BuiltinResult::Status(jobs::bg_cmd(argv, shell_env, stderr)?),
        "source" | "." => source::source(argv, shell_env, stderr)?,
        "@" => BuiltinResult::Status(at::at_cmd(argv, shell_env, stderr)?),
        "exit" => exit::exit_cmd(argv, last_status, stderr)?,
        _ => return Ok(None),
    };

    Ok(Some(result))
}
