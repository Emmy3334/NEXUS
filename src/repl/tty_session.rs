//! Interactive TTY startup: layered files then session histfile.

use super::history_persist;
use super::newuser;
use super::rc::{self, RcLoad};
use crate::env::boot_load;
use crate::env::ShellEnvironment;

use std::io::{self, BufRead, Write};

/// Outcome of preparing an interactive TTY session.
pub(super) enum Boot {
    /// A startup file called `exit`.
    Exit(u8),
    /// Ready to enter the REPL with this initial status.
    Ready(u8),
}

/// Load startup chain + histfile when `tty`; otherwise leave status at 0.
pub(super) fn boot<I: BufRead, O: Write, E: Write>(
    tty: bool,
    login: bool,
    shell_env: &mut ShellEnvironment,
    stdin: &mut I,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<Boot> {
    if !tty {
        return Ok(Boot::Ready(0));
    }
    // zsh/newuser analogue: offer setup before first `.nexusrc` exists.
    newuser::maybe_run(stdin, stdout, stderr, shell_env)?;
    // Real TTY only: Cursor-based interactive tests must not pick up ~/.nexusrc.
    let status = match rc::load_startup_chain(tty, login, shell_env, stdout, stderr)? {
        RcLoad::Exit(code) => return Ok(Boot::Exit(code)),
        RcLoad::Continue(code) => code,
        RcLoad::Skipped => 0,
    };
    // After RC so `set histfile=…` is honored; missing file is quiet.
    history_persist::load_session_history(shell_env, stderr)?;
    boot_load(shell_env);
    Ok(Boot::Ready(status))
}
