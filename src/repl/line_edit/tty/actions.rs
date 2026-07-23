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
    var_names: &[String],
) -> io::Result<Loop> {
    match action {
        Action::Accept => {
            edit.complete_cycle = None;
            Ok(Loop::Accept)
        }
        Action::Eof => Ok(Loop::Eof),
        Action::Interrupt => {
            edit.complete_cycle = None;
            Ok(Loop::Interrupt)
        }
        Action::Complete => complete_token(stdout, edit, prompt, var_names),
        other => {
            edit.complete_cycle = None;
            apply_other(stdout, edit, bindings, other, prompt, nav)
        }
    }
}

fn apply_other(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    bindings: &mut KeyBindings,
    action: Action,
    prompt: &str,
    nav: &mut HistoryRecall<'_>,
) -> io::Result<Loop> {
    match action {
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
        Action::ClearScreen => {
            draw::clear_screen(stdout)?;
            draw::redraw(stdout, prompt, edit)?;
            Ok(Loop::Continue)
        }
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
        Action::MoveHome => buffer::move_home(edit),
        Action::MoveEnd => buffer::move_end(edit),
        Action::MoveWordLeft => buffer::move_word_left(edit),
        Action::MoveWordRight => buffer::move_word_right(edit),
        Action::KillWordForward => buffer::kill_word_forward(edit),
        Action::KillWordBackward => buffer::kill_word_backward(edit),
        Action::KillToEol => buffer::kill_to_eol(edit),
        Action::KillLine => buffer::kill_line(edit),
        Action::Yank => buffer::yank(edit),
        Action::TransposeWords => buffer::transpose_words(edit),
        _ => {}
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn complete_token(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &str,
    var_names: &[String],
) -> io::Result<Loop> {
    let matches = complete::complete_or_cycle(
        &mut edit.text,
        &mut edit.cursor,
        var_names,
        &mut edit.complete_cycle,
    );
    let lines = complete::list_display_lines(&matches);
    if !lines.is_empty() {
        writeln!(stdout)?;
    }
    for line in &lines {
        writeln!(stdout, "{line}")?;
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}
