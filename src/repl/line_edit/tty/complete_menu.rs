//! Arrow menu-select while an ambiguous Tab cycle is active.

use super::actions::Loop;
use super::buffer::EditBuffer;
use super::draw;
use crate::keybind::Action;
use crate::repl::line_edit::complete;
use crate::repl::line_edit::complete::CompleteCtx;

use std::io::{self, Write};

/// Handle menu keys; `None` means the action is not for the menu.
pub(super) fn on_action(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    action: Action,
    prompt: &str,
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Option<Loop>> {
    if action == Action::Complete {
        return Ok(Some(on_tab(stdout, edit, prompt, complete_ctx)?));
    }
    let active = edit
        .complete_cycle
        .as_ref()
        .is_some_and(|c| c.is_active(&edit.text, edit.cursor));
    if !active {
        return Ok(None);
    }
    match action {
        Action::HistoryUp => Ok(Some(move_hl(stdout, edit, prompt, true)?)),
        Action::HistoryDown => Ok(Some(move_hl(stdout, edit, prompt, false)?)),
        Action::Accept => Ok(Some(accept_item(stdout, edit, prompt)?)),
        Action::Interrupt | Action::ViCmdMode => Ok(Some(cancel(stdout, edit, prompt)?)),
        _ => {
            edit.complete_cycle = None;
            Ok(None)
        }
    }
}

fn on_tab(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &str,
    complete_ctx: &CompleteCtx<'_>,
) -> io::Result<Loop> {
    let listed = complete::complete_or_cycle(
        &mut edit.text,
        &mut edit.cursor,
        complete_ctx,
        &mut edit.complete_cycle,
    );
    if listed.is_empty() {
        if edit.complete_cycle.is_some() {
            repaint(stdout, edit, prompt)?;
        } else {
            draw::redraw(stdout, prompt, edit)?;
        }
        return Ok(Loop::Continue);
    }
    paint_new(stdout, edit, prompt)
}

fn move_hl(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    prompt: &str,
    up: bool,
) -> io::Result<Loop> {
    if let Some(cycle) = edit.complete_cycle.as_mut() {
        if up {
            cycle.move_up();
        } else {
            cycle.move_down();
        }
    }
    repaint(stdout, edit, prompt)?;
    Ok(Loop::Continue)
}

fn accept_item(stdout: &mut impl Write, edit: &mut EditBuffer, prompt: &str) -> io::Result<Loop> {
    if let Some(cycle) = edit.complete_cycle.take() {
        cycle.accept(&mut edit.text, &mut edit.cursor);
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn cancel(stdout: &mut impl Write, edit: &mut EditBuffer, prompt: &str) -> io::Result<Loop> {
    edit.complete_cycle = None;
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn paint_new(stdout: &mut impl Write, edit: &mut EditBuffer, prompt: &str) -> io::Result<Loop> {
    let Some(cycle) = edit.complete_cycle.as_ref() else {
        return draw::redraw(stdout, prompt, edit).map(|_| Loop::Continue);
    };
    let lines = complete::list_menu_lines_tagged(&cycle.matches, cycle.highlight);
    writeln!(stdout)?;
    for line in &lines {
        writeln!(stdout, "{line}")?;
    }
    if let Some(cycle) = edit.complete_cycle.as_mut() {
        cycle.list_rows = lines.len();
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn repaint(stdout: &mut impl Write, edit: &mut EditBuffer, prompt: &str) -> io::Result<()> {
    let Some(cycle) = edit.complete_cycle.as_ref() else {
        return draw::redraw(stdout, prompt, edit);
    };
    let rows = cycle.list_rows;
    let lines = complete::list_menu_lines_tagged(&cycle.matches, cycle.highlight);
    if rows > 0 {
        draw::shift_rows(stdout, -(rows as i32))?;
    }
    for line in &lines {
        write!(stdout, "\r\x1b[2K{line}\n")?;
    }
    if let Some(cycle) = edit.complete_cycle.as_mut() {
        cycle.list_rows = lines.len();
    }
    draw::redraw(stdout, prompt, edit)
}
