//! Interactive TTY startup: `~/.nexusrc` then session histfile.

use super::history_persist;
use super::rc::{self, RcLoad};
use crate::env::ShellEnvironment;

use std::io::{self, Write};

/// Outcome of preparing an interactive TTY session.
pub(super) enum Boot {
    /// RC called `exit`.
    Exit(u8),
    /// Ready to enter the REPL with this initial status.
    Ready(u8),
}

/// Load RC + histfile when `tty`; otherwise leave status at 0.
pub(super) fn boot<O: Write, E: Write>(
    tty: bool,
    shell_env: &mut ShellEnvironment,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<Boot> {
    if !tty {
        return Ok(Boot::Ready(0));
    }
    // Real TTY only: Cursor-based interactive tests must not pick up ~/.nexusrc.
    let status = match rc::load_startup_rc(shell_env, stdout, stderr)? {
        RcLoad::Exit(code) => return Ok(Boot::Exit(code)),
        RcLoad::Continue(code) => code,
        RcLoad::Skipped => 0,
    };
    // After RC so `set histfile=…` is honored; missing file is quiet.
    history_persist::load_session_history(shell_env, stderr)?;
    Ok(Boot::Ready(status))
}
