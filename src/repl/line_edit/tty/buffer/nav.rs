//! Cursor motion and delete helpers for [`super::EditBuffer`].

use super::EditBuffer;

pub(in crate::repl::line_edit::tty) fn backspace(edit: &mut EditBuffer) {
    if edit.cursor == 0 {
        return;
    }
    let prev = prev_boundary(&edit.text, edit.cursor);
    edit.text.drain(prev..edit.cursor);
    edit.cursor = prev;
}

pub(in crate::repl::line_edit::tty) fn delete(edit: &mut EditBuffer) {
    if edit.cursor >= edit.text.len() {
        return;
    }
    let next = next_boundary(&edit.text, edit.cursor);
    edit.text.drain(edit.cursor..next);
}

pub(in crate::repl::line_edit::tty) fn move_left(edit: &mut EditBuffer) {
    if edit.cursor > 0 {
        edit.cursor = prev_boundary(&edit.text, edit.cursor);
    }
}

pub(in crate::repl::line_edit::tty) fn move_right(edit: &mut EditBuffer) {
    if edit.cursor < edit.text.len() {
        edit.cursor = next_boundary(&edit.text, edit.cursor);
    }
}

fn prev_boundary(text: &str, index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    let mut i = index - 1;
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_boundary(text: &str, index: usize) -> usize {
    if index >= text.len() {
        return text.len();
    }
    let mut i = index + 1;
    while i < text.len() && !text.is_char_boundary(i) {
        i += 1;
    }
    i
}
