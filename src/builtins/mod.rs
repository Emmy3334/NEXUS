//! Shell builtins: Minishell1 plus 42sh locals, aliases, history, and jobs.
//!
//! Each command lives in its own module; this file dispatches by `argv[0]`.

mod alias;
mod at;
mod bindkey;
mod cd;
mod comp;
mod dirstack;
mod dispatch;
mod docker;
mod echo_cmd;
mod env;
mod exit;
mod hash_cmd;
mod heal;
mod history;
mod jobs;
mod kube;
mod local_cmd;
mod names;
mod repeat;
mod return_cmd;
mod sandbox;
mod set;
mod setenv;
mod source;
mod trivial;
mod typeset;
mod unalias;
mod unset;
mod unsetenv;
mod which;

pub use names::{is_builtin, NAMES};

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
