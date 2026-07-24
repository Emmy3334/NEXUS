//! Interactive TTY line editor (raw mode).

mod actions;
mod buffer;
mod complete_menu;
mod draw;
mod escape;
mod event;
mod handle;
mod isearch_mode;
mod keys;
mod queue;
mod session;
mod term;

pub use buffer::{after_line_down, after_line_up};
pub use queue::take_complete_line;

use super::ReadOutcome;
use crate::history::History;
use crate::keybind::KeyBindings;
use crate::repl::line_edit::complete::CompleteCtx;

use std::collections::VecDeque;
use std::io::{self, Write};

pub(super) fn edit_line(
    stdout: &mut impl Write,
    out: &mut String,
    history: &History,
    bindings: &mut KeyBindings,
    queue: &mut VecDeque<u8>,
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<ReadOutcome> {
    // Multi-line paste leftovers: already shown under one prompt; run silently.
    if let Some(line) = take_complete_line(queue) {
        *out = line;
        return Ok(ReadOutcome::Line);
    }
    let _guard = term::RawMode::enter()?;
    session::run(stdout, out, history, bindings, queue, complete_ctx)
}
