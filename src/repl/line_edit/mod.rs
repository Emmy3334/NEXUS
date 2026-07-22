//! Physical / logical line acquisition for the REPL.

mod complete;
mod continue_line;
mod input;
mod plain;
mod probe;
mod recall;
#[cfg(unix)]
mod tty;

pub use crate::keybind::{Action, KeyBindings};
pub use input::ReplInput;
pub use recall::HistoryRecall;
#[cfg(unix)]
pub use tty::take_complete_line;

use super::prompt;
use crate::history::History;
use crate::keybind::KeyBindings as Bindings;
use std::collections::VecDeque;
use std::io::{self, Write};

/// Read one logical command line (may span physical lines when quotes are open).
pub(super) fn read_logical_line(
    stdin: &mut impl ReplInput,
    stdout: &mut impl Write,
    interactive: bool,
    history: &History,
    bindings: &mut Bindings,
    buffer: &mut String,
    queue: &mut VecDeque<u8>,
) -> io::Result<ReadOutcome> {
    buffer.clear();
    if interactive && stdin.is_terminal() {
        return read_tty(stdout, buffer, history, bindings, queue);
    }
    prompt::write_primary(stdout, interactive)?;
    if !plain::read_into(stdin, buffer, queue)? {
        return Ok(ReadOutcome::Eof);
    }
    if interactive {
        continue_line::join_until_closed(stdin, stdout, buffer, queue)?;
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
    bindings: &mut Bindings,
    queue: &mut VecDeque<u8>,
) -> io::Result<ReadOutcome> {
    #[cfg(unix)]
    {
        tty::edit_line(stdout, buffer, history, bindings, queue)
    }
    #[cfg(not(unix))]
    {
        let _ = (stdout, buffer, history, bindings, queue);
        Ok(ReadOutcome::Eof)
    }
}
