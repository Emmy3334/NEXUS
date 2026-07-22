//! Parse and resolve a single `!` history reference.

use super::cursor::{bump, peek};
use super::event::take_event;
use super::modifier::apply::apply_modifiers;
use super::scan::is_literal_bang_follower;
use super::words::{select_words, take_word_sel};
use super::HistoryError;
use crate::history::History;

/// Result text and whether `:p` requested print-only.
pub(super) struct DesignatorOut {
    pub(super) text: String,
    pub(super) print_only: bool,
    /// False when `!` was left literal (blank / `=` / `(` follower).
    pub(super) expanded: bool,
}

/// Consume a designator starting at `s[*i] == '!'`.
pub(super) fn expand_designator(
    s: &str,
    i: &mut usize,
    history: &mut History,
    current: &str,
) -> Result<DesignatorOut, HistoryError> {
    bump(s, i); // skip '!'
    if peek(s, *i).map_or(true, is_literal_bang_follower) {
        return Ok(DesignatorOut {
            text: "!".into(),
            print_only: false,
            expanded: false,
        });
    }
    let event = take_event(s, i, history, current)?;
    let sel = take_word_sel(s, i)?;
    let selected = select_words(&event.text, sel, history)?;
    let result = apply_modifiers(s, i, selected, history)?;
    Ok(DesignatorOut {
        text: result.text,
        print_only: result.print_only,
        expanded: true,
    })
}
