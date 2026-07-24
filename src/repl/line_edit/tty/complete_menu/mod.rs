//! Arrow menu-select while an ambiguous Tab cycle is active.
//!
//! The completion menu is painted **below** the input line; the cursor stays on
//! the prompt row until Enter (no forced fresh prompt from Tab alone).

mod paint;

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
            dismiss_menu(stdout, edit)?;
            Ok(None)
        }
    }
}

/// Drop any open menu and erase its TTY rows (call before other redraws).
pub(super) fn dismiss_menu(stdout: &mut impl Write, edit: &mut EditBuffer) -> io::Result<()> {
    let rows = edit.complete_cycle.take().map(|c| c.list_rows).unwrap_or(0);
    draw::erase_rows_below(stdout, rows)
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
    if listed.is_empty() && edit.complete_cycle.is_none() {
        draw::redraw(stdout, prompt, edit)?;
        return Ok(Loop::Continue);
    }
    paint::show(stdout, edit, prompt)?;
    Ok(Loop::Continue)
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
    paint::show(stdout, edit, prompt)?;
    Ok(Loop::Continue)
}

fn accept_item(stdout: &mut impl Write, edit: &mut EditBuffer, prompt: &str) -> io::Result<Loop> {
    if let Some(cycle) = edit.complete_cycle.take() {
        draw::erase_rows_below(stdout, cycle.list_rows)?;
        cycle.accept(&mut edit.text, &mut edit.cursor);
    }
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}

fn cancel(stdout: &mut impl Write, edit: &mut EditBuffer, prompt: &str) -> io::Result<Loop> {
    dismiss_menu(stdout, edit)?;
    draw::redraw(stdout, prompt, edit)?;
    Ok(Loop::Continue)
}
