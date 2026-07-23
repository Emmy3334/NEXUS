//! Redraw the current visual line on the TTY.

use super::buffer::EditBuffer;
use std::io::{self, Write};

pub(super) fn clear_screen(stdout: &mut impl Write) -> io::Result<()> {
    write!(stdout, "\x1b[H\x1b[2J")
}

pub(super) fn redraw(stdout: &mut impl Write, prompt: &str, edit: &EditBuffer) -> io::Result<()> {
    let (line, cursor_in_line) = visible_line(edit);
    write!(stdout, "\r\x1b[2K{prompt}{line}")?;
    let after = line.len().saturating_sub(cursor_in_line);
    if after > 0 {
        write!(stdout, "\x1b[{after}D")?;
    }
    stdout.flush()
}

fn visible_line(edit: &EditBuffer) -> (&str, usize) {
    let text = edit.as_str();
    let cursor = edit.cursor;
    let start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[cursor..]
        .find('\n')
        .map(|i| cursor + i)
        .unwrap_or(text.len());
    (&text[start..end], cursor - start)
}
