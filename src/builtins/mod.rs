//! Shell builtins: Minishell1 plus 42sh locals, aliases, history, and jobs.
//!
//! Each command lives in its own module; this file dispatches by `argv[0]`.

mod alias;
mod at;
mod bindkey;
mod cd;
mod dirstack;
mod dispatch;
mod env;
mod exit;
mod history;
mod jobs;
mod kube;
mod repeat;
mod return_cmd;
mod sandbox;
mod set;
mod setenv;
mod source;
mod unalias;
mod unset;
mod unsetenv;
mod which;

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
    /// Run `argv` `count` times (handled by exec to avoid cycles).
    Repeat { count: u32, argv: Vec<String> },
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
            | "which"
            | "where"
            | "repeat"
            | "sandbox"
            | "@kube"
            | "pushd"
            | "popd"
            | "dirs"
            | "return"
            | "disown"
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
    dispatch::dispatch(name, argv, shell_env, last_status, stdout, stderr)
}
