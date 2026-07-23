//! Kill-ring operations for the TTY edit buffer.

use super::word::{backward_word_start, forward_word_end};
use super::EditBuffer;

const KILL_RING_CAP: usize = 10;

pub(in crate::repl::line_edit::tty) fn kill_word_forward(edit: &mut EditBuffer) {
    let end = forward_word_end(&edit.text, edit.cursor);
    take_range(edit, edit.cursor, end);
}

pub(in crate::repl::line_edit::tty) fn kill_word_backward(edit: &mut EditBuffer) {
    let start = backward_word_start(&edit.text, edit.cursor);
    take_range(edit, start, edit.cursor);
}

pub(in crate::repl::line_edit::tty) fn kill_to_eol(edit: &mut EditBuffer) {
    take_range(edit, edit.cursor, edit.text.len());
}

pub(in crate::repl::line_edit::tty) fn kill_line(edit: &mut EditBuffer) {
    take_range(edit, 0, edit.text.len());
}

pub(in crate::repl::line_edit::tty) fn yank(edit: &mut EditBuffer) {
    let Some(text) = edit.kill_ring.front().cloned() else {
        return;
    };
    for ch in text.chars() {
        edit.insert(ch);
    }
}

fn take_range(edit: &mut EditBuffer, start: usize, end: usize) {
    if start >= end {
        return;
    }
    let killed: String = edit.text.drain(start..end).collect();
    push_kill(edit, killed);
    edit.cursor = start;
}

fn push_kill(edit: &mut EditBuffer, text: String) {
    if text.is_empty() {
        return;
    }
    edit.kill_ring.push_front(text);
    while edit.kill_ring.len() > KILL_RING_CAP {
        edit.kill_ring.pop_back();
    }
}
