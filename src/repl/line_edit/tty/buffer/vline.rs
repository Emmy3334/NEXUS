//! Vertical cursor motion across `\n` in the edit buffer.

use super::EditBuffer;

/// Byte index after moving up one physical line, if not already on the first.
#[must_use]
pub fn after_line_up(text: &str, cursor: usize) -> Option<usize> {
    let cursor = cursor.min(text.len());
    let line_start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    if line_start == 0 {
        return None;
    }
    let col = text[line_start..cursor].chars().count();
    let prev_end = line_start - 1;
    let prev_start = text[..prev_end].rfind('\n').map(|i| i + 1).unwrap_or(0);
    Some(offset_at_col(text, prev_start, prev_end, col))
}

/// Byte index after moving down one physical line, if a lower line exists.
#[must_use]
pub fn after_line_down(text: &str, cursor: usize) -> Option<usize> {
    let cursor = cursor.min(text.len());
    let line_start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[cursor..]
        .find('\n')
        .map(|i| cursor + i)
        .unwrap_or(text.len());
    if line_end >= text.len() {
        return None;
    }
    let col = text[line_start..cursor].chars().count();
    let next_start = line_end + 1;
    let next_end = text[next_start..]
        .find('\n')
        .map(|i| next_start + i)
        .unwrap_or(text.len());
    Some(offset_at_col(text, next_start, next_end, col))
}

pub(in crate::repl::line_edit::tty) fn move_line_up(edit: &mut EditBuffer) -> bool {
    match after_line_up(&edit.text, edit.cursor) {
        Some(pos) => {
            edit.cursor = pos;
            true
        }
        None => false,
    }
}

pub(in crate::repl::line_edit::tty) fn move_line_down(edit: &mut EditBuffer) -> bool {
    match after_line_down(&edit.text, edit.cursor) {
        Some(pos) => {
            edit.cursor = pos;
            true
        }
        None => false,
    }
}

fn offset_at_col(text: &str, start: usize, end: usize, col: usize) -> usize {
    let line = &text[start..end];
    for (n, (i, _)) in line.char_indices().enumerate() {
        if n == col {
            return start + i;
        }
    }
    end
}
