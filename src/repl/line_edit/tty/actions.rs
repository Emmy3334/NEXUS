//! Dispatch bound key actions for the TTY editor.

use super::buffer::{self, EditBuffer};
use super::draw;
use crate::keybind::{Action, KeyBindings};
use crate::repl::line_edit::complete;
use crate::repl::line_edit::recall::HistoryRecall;
use crate::repl::prompt;

use std::io::{self, Write};

pub(super) enum Loop {
    Continue,
    Accept,
    Eof,
    Interrupt,
}

pub(super) fn apply(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    action: Action,
    prompt: &str,
    nav: &mut HistoryRecall<'_>,
) -> io::Result<Loop> {
    match action {
        Action::Accept => Ok(Loop::Accept),
        Action::Eof => Ok(Loop::Eof),
        Action::Interrupt => Ok(Loop::Interrupt),
        Action::ViCmdMode => {
            bindings.enter_command_map();
            Ok(Loop::Continue)
        }
        Action::ViInsertMode => {
            bindings.enter_insert_map();
            Ok(Loop::Continue)
        }
        Action::HistoryUp if prompt::is_primary(prompt) => {
            apply_recall(stdout, edit, prompt, |n, line| n.older(line), nav)
        }
        Action::HistoryDown if prompt::is_primary(prompt) => {
            apply_recall(stdout, edit, prompt, |n, line| n.newer(line), nav)
        }
        Action::HistoryUp | Action::HistoryDown => Ok(Loop::Continue),
        Action::Complete => complete_token(stdout, edit, prompt),
        other => mutate(stdout, edit, other, prompt),
    }
}

fn apply_recall(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &str,
    step: impl FnOnce(&mut HistoryRecall<'_>, &mut String),
    nav: &mut HistoryRecall<'_>,
) -> io::Result<Loop> {
    step(nav, &mut edit.text);
    edit.cursor = edit.text.len();
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn mutate(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    action: Action,
    prompt: &str,
) -> io::Result<Loop> {
    match action {
        Action::Backspace => buffer::backspace(edit),
        Action::Delete => buffer::delete(edit),
        Action::MoveLeft => buffer::move_left(edit),
        Action::MoveRight => buffer::move_right(edit),
        _ => {}
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn complete_token(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &str,
) -> io::Result<Loop> {
    let matches = complete::complete(&mut edit.text, &mut edit.cursor);
    if !matches.is_empty() {
        writeln!(stdout)?;
    }
    for item in &matches {
        writeln!(stdout, "{item}")?;
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}
