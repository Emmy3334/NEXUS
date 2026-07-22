//! Interactive step helpers: precmd and ignoreeof.

use super::line_edit::ReplInput;
use super::ReplIo;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run_precmd_if_interactive<I: ReplInput, O: Write, E: Write>(
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ReplIo<'_, I, O, E>,
) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    crate::specials::run_precmd(shell_env, last_status, io.stdin, io.stdout, io.stderr)
}

/// `Ok(true)` means ignore this EOF and continue the REPL loop.
pub(super) fn suppress_eof(
    interactive: bool,
    is_eof: bool,
    shell_env: &ShellEnvironment,
    eof_streak: &mut u32,
    stderr: &mut impl Write,
) -> io::Result<bool> {
    if !is_eof {
        *eof_streak = 0;
        return Ok(false);
    }
    if !interactive {
        return Ok(false);
    }
    let exit = crate::specials::allow_exit_on_eof(shell_env, eof_streak, stderr)?;
    Ok(!exit)
}
