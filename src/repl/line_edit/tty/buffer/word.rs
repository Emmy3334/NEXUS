//! Word-boundary helpers (whitespace-separated words).

use super::EditBuffer;

#[must_use]
pub(super) fn forward_word_end(text: &str, mut index: usize) -> usize {
    let bytes = text.as_bytes();
    while index < bytes.len() && is_ws(bytes[index]) {
        index = next(text, index);
    }
    while index < bytes.len() && !is_ws(bytes[index]) {
        index = next(text, index);
    }
    index
}

#[must_use]
pub(super) fn backward_word_start(text: &str, mut index: usize) -> usize {
    while index > 0 {
        let prev = prev(text, index);
        if !is_ws(text.as_bytes()[prev]) {
            break;
        }
        index = prev;
    }
    while index > 0 {
        let prev = prev(text, index);
        if is_ws(text.as_bytes()[prev]) {
            break;
        }
        index = prev;
    }
    index
}

pub(in crate::repl::line_edit::tty) fn move_word_left(edit: &mut EditBuffer) {
    edit.cursor = backward_word_start(&edit.text, edit.cursor);
}

pub(in crate::repl::line_edit::tty) fn move_word_right(edit: &mut EditBuffer) {
    edit.cursor = forward_word_end(&edit.text, edit.cursor);
}

fn is_ws(b: u8) -> bool {
    b.is_ascii_whitespace()
}

fn prev(text: &str, index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    let mut i = index - 1;
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next(text: &str, index: usize) -> usize {
    if index >= text.len() {
        return text.len();
    }
    let mut i = index + 1;
    while i < text.len() && !text.is_char_boundary(i) {
        i += 1;
    }
    i
}
