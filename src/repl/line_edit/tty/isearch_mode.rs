//! Handle key events while reverse-i-search is active.

use super::actions::Loop;
use super::buffer::EditBuffer;
use super::draw;
use crate::history::History;
use crate::keybind::Action;
use crate::repl::line_edit::isearch::HistoryISearch;

use std::io::{self, Write};

pub(super) fn sync(edit: &mut EditBuffer, search: &HistoryISearch<'_>) {
    edit.complete_cycle = None;
    edit.text.clear();
    edit.text.push_str(search.display_line());
    edit.cursor = edit.text.len();
}

pub(super) fn redraw(
    stdout: &mut impl Write,
    edit: &EditBuffer,
    search: &HistoryISearch<'_>,
) -> io::Result<()> {
    draw::redraw(stdout, &search.prompt_label(), edit)
}

pub(super) fn begin<'a>(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    history: &'a History,
    isearch: &mut Option<HistoryISearch<'a>>,
) -> io::Result<Loop> {
    let search = HistoryISearch::start(history, edit.text.clone());
    sync(edit, &search);
    redraw(stdout, edit, &search)?;
    *isearch = Some(search);
    Ok(Loop::Continue)
}

pub(super) fn on_action(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    isearch: &mut Option<HistoryISearch<'_>>,
    action: Action,
) -> io::Result<Option<Loop>> {
    let Some(active) = isearch.as_mut() else {
        return Ok(None);
    };
    match action {
        Action::HistoryISearch => {
            active.again();
            sync(edit, active);
            redraw(stdout, edit, active)?;
            Ok(Some(Loop::Continue))
        }
        Action::Backspace => {
            active.backspace();
            sync(edit, active);
            redraw(stdout, edit, active)?;
            Ok(Some(Loop::Continue))
        }
        Action::Accept => {
            sync(edit, active);
            *isearch = None;
            Ok(Some(Loop::Accept))
        }
        Action::Interrupt | Action::Eof => {
            abort(edit, isearch);
            redraw_primary(stdout, edit)?;
            Ok(Some(Loop::Continue))
        }
        _ => {
            // Leave isearch with the current match (zsh-like) and fall through.
            sync(edit, active);
            *isearch = None;
            Ok(None)
        }
    }
}

pub(super) fn on_insert(
    stdout: &mut impl Write,
    edit: &mut EditBuffer,
    search: &mut HistoryISearch<'_>,
    text: &str,
) -> io::Result<Loop> {
    for ch in text.chars() {
        search.push_char(ch);
    }
    sync(edit, search);
    redraw(stdout, edit, search)?;
    Ok(Loop::Continue)
}

pub(super) fn abort(edit: &mut EditBuffer, search: &mut Option<HistoryISearch<'_>>) {
    if let Some(active) = search.take() {
        edit.text.clear();
        edit.text.push_str(active.draft());
        edit.cursor = edit.text.len();
    }
}

pub(super) fn redraw_primary(stdout: &mut impl Write, edit: &EditBuffer) -> io::Result<()> {
    draw::redraw(stdout, &crate::repl::prompt::format_primary(), edit)
}
