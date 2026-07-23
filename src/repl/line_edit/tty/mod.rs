//! Interactive TTY line editor (raw mode).

mod actions;
mod buffer;
mod draw;
mod escape;
mod event;
mod handle;
mod isearch_mode;
mod keys;
mod queue;
mod term;

pub use queue::take_complete_line;

use self::actions::Loop;
use self::buffer::EditBuffer;
use super::isearch::HistoryISearch;
use super::probe::quotes_closed;
use super::recall::HistoryRecall;
use super::ReadOutcome;
use crate::history::History;
use crate::keybind::KeyBindings;
use crate::repl::prompt::PromptLine;

use std::collections::VecDeque;
use std::io::{self, Write};

pub(super) fn edit_line(
    stdout: &mut impl Write,
    out: &mut String,
    history: &History,
    bindings: &mut KeyBindings,
    queue: &mut VecDeque<u8>,
) -> io::Result<ReadOutcome> {
    let _guard = term::RawMode::enter()?;
    let mut edit = EditBuffer::new();
    let mut nav = HistoryRecall::new(history);
    let mut prompt = PromptLine::primary();
    let mut pasting = false;
    let mut isearch: Option<HistoryISearch<'_>> = None;
    bindings.enter_insert_map();
    draw::redraw(stdout, prompt.as_str(), &edit)?;
    loop {
        match handle::handle_event(
            stdout,
            &mut edit,
            bindings,
            &mut prompt,
            &mut nav,
            queue,
            &mut pasting,
            history,
            &mut isearch,
        )? {
            Loop::Continue => {}
            Loop::Accept => {
                isearch = None;
                if finish_accept(stdout, &mut edit, &mut prompt, out)? {
                    return Ok(ReadOutcome::Line);
                }
                nav = HistoryRecall::new(history);
            }
            Loop::Eof if edit.is_empty() && isearch.is_none() => {
                out.clear();
                return Ok(ReadOutcome::Eof);
            }
            Loop::Eof => {}
            Loop::Interrupt => {
                isearch = None;
                on_interrupt(stdout, &mut edit, &mut nav, history, &mut prompt)?;
            }
        }
    }
}

fn on_interrupt<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    nav: &mut HistoryRecall<'a>,
    history: &'a History,
    prompt: &mut PromptLine,
) -> io::Result<()> {
    edit.clear();
    *nav = HistoryRecall::new(history);
    writeln!(stdout, "^C")?;
    prompt.set_primary();
    draw::redraw(stdout, prompt.as_str(), edit)
}

fn finish_accept(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &mut PromptLine,
    out: &mut String,
) -> io::Result<bool> {
    writeln!(stdout)?;
    if quotes_closed(edit.as_str()) {
        *out = std::mem::take(&mut edit.text);
        edit.cursor = 0;
        return Ok(true);
    }
    edit.push_char('\n');
    prompt.set_continue();
    write!(stdout, "{}", prompt.as_str())?;
    stdout.flush()?;
    Ok(false)
}
