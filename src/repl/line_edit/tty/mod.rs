//! Interactive TTY line editor (raw mode).

mod actions;
mod buffer;
mod draw;
mod escape;
mod event;
mod keys;
mod queue;
mod term;

pub use queue::take_complete_line;

use self::actions::Loop;
use self::buffer::EditBuffer;
use self::event::Event;
use self::keys::read_event;
use super::bindings::KeyBindings;
use super::probe::quotes_closed;
use super::recall::HistoryRecall;
use super::ReadOutcome;
use crate::history::History;
use crate::repl::prompt;

use std::collections::VecDeque;
use std::io::{self, Write};

pub(super) fn edit_line(
    stdout: &mut impl Write,
    out: &mut String,
    history: &History,
    queue: &mut VecDeque<u8>,
) -> io::Result<ReadOutcome> {
    let _guard = term::RawMode::enter()?;
    let bindings = KeyBindings::default();
    let mut edit = EditBuffer::new();
    let mut nav = HistoryRecall::new(history);
    let mut prompt = prompt::PRIMARY;
    draw::redraw(stdout, prompt, &edit)?;
    loop {
        match handle_event(stdout, &mut edit, &bindings, &mut prompt, &mut nav, queue)? {
            Loop::Continue => {}
            Loop::Accept => {
                if finish_accept(stdout, &mut edit, &mut prompt, out)? {
                    return Ok(ReadOutcome::Line);
                }
                nav = HistoryRecall::new(history);
            }
            Loop::Eof if edit.is_empty() => {
                out.clear();
                return Ok(ReadOutcome::Eof);
            }
            Loop::Eof => {}
            Loop::Interrupt => on_interrupt(stdout, &mut edit, &mut nav, history, &mut prompt)?,
        }
    }
}

fn on_interrupt<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    nav: &mut HistoryRecall<'a>,
    history: &'a History,
    prompt: &mut &str,
) -> io::Result<()> {
    edit.clear();
    *nav = HistoryRecall::new(history);
    writeln!(stdout, "^C")?;
    *prompt = prompt::PRIMARY;
    draw::redraw(stdout, prompt, edit)
}

fn finish_accept(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &mut &str,
    out: &mut String,
) -> io::Result<bool> {
    writeln!(stdout)?;
    if quotes_closed(edit.as_str()) {
        *out = std::mem::take(&mut edit.text);
        edit.cursor = 0;
        return Ok(true);
    }
    edit.push_char('\n');
    *prompt = prompt::CONTINUE;
    write!(stdout, "{prompt}")?;
    stdout.flush()?;
    Ok(false)
}

fn handle_event(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &KeyBindings,
    prompt: &mut &str,
    nav: &mut HistoryRecall<'_>,
    queue: &mut VecDeque<u8>,
) -> io::Result<Loop> {
    match read_event(queue)? {
        Event::Action(action) => actions::apply(stdout, edit, action, prompt, nav),
        Event::InsertRun(text) => {
            for ch in text.chars() {
                edit.insert(ch);
            }
            draw::redraw(stdout, prompt, edit)?;
            Ok(Loop::Continue)
        }
        Event::Raw(bytes) => {
            if let Some(action) = bindings.lookup(&bytes) {
                actions::apply(stdout, edit, action, prompt, nav)
            } else {
                Ok(Loop::Continue)
            }
        }
    }
}
