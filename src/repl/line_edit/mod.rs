//! Physical / logical line acquisition for the REPL.

mod bindings;
mod complete;
mod continue_line;
mod input;
mod plain;
mod probe;
mod recall;
#[cfg(unix)]
mod tty;

pub use bindings::{Action, KeyBindings};
pub use input::ReplInput;
pub use recall::HistoryRecall;

use super::prompt;
use crate::history::History;
use std::io::{self, Write};

/// Read one logical command line (may span physical lines when quotes are open).
pub(super) fn read_logical_line(
    stdin: &mut impl ReplInput,
    stdout: &mut impl Write,
    interactive: bool,
    history: &History,
    buffer: &mut String,
) -> io::Result<ReadOutcome> {
    buffer.clear();
    if interactive && stdin.is_terminal() {
        return read_tty(stdout, buffer, history);
    }
    prompt::write_primary(stdout, interactive)?;
    if !plain::read_into(stdin, buffer)? {
        return Ok(ReadOutcome::Eof);
    }
    if interactive {
        continue_line::join_until_closed(stdin, stdout, buffer)?;
    }
    Ok(ReadOutcome::Line)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReadOutcome {
    Eof,
    Line,
}

fn read_tty(
    stdout: &mut impl Write,
    buffer: &mut String,
    history: &History,
) -> io::Result<ReadOutcome> {
    #[cfg(unix)]
    {
        tty::edit_line(stdout, buffer, history)
    }
    #[cfg(not(unix))]
    {
        let _ = stdout;
        let _ = buffer;
        let _ = history;
        Ok(ReadOutcome::Eof)
    }
}
