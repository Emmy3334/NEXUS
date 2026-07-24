//! Redraw the current visual line on the TTY.

use super::buffer::EditBuffer;
use crate::repl::prompt;
use std::io::{self, Write};

pub(super) fn clear_screen(stdout: &mut impl Write) -> io::Result<()> {
    write!(stdout, "\x1b[H\x1b[2J")
}

pub(super) fn redraw(
    stdout: &mut impl Write,
    prompt_str: &str,
    edit: &EditBuffer,
) -> io::Result<()> {
    let (line, cursor_in_line, continuation) = visible_line(edit);
    // Paste / multi-line under PS1: bare continuation rows (zsh-like). PS2 still repeats.
    let shown = if continuation && prompt::is_primary(prompt_str) {
        ""
    } else {
        prompt_str
    };
    write!(stdout, "\r\x1b[2K{shown}{line}")?;
    let after = line.len().saturating_sub(cursor_in_line);
    if after > 0 {
        write!(stdout, "\x1b[{after}D")?;
    }
    stdout.flush()
}

/// Move the terminal cursor by `delta` rows (negative = up) before a redraw.
pub(super) fn shift_rows(stdout: &mut impl Write, delta: i32) -> io::Result<()> {
    if delta < 0 {
        write!(stdout, "\x1b[{}A", -delta)?;
    } else if delta > 0 {
        write!(stdout, "\x1b[{delta}B")?;
    }
    Ok(())
}

/// Clear `rows` lines below the current cursor row, then return to this row.
pub(super) fn erase_rows_below(stdout: &mut impl Write, rows: usize) -> io::Result<()> {
    if rows == 0 {
        return Ok(());
    }
    shift_rows(stdout, 1)?;
    for _ in 0..rows {
        write!(stdout, "\r\x1b[2K\n")?;
    }
    shift_rows(stdout, -((rows + 1) as i32))
}

fn visible_line(edit: &EditBuffer) -> (&str, usize, bool) {
    let text = edit.as_str();
    let cursor = edit.cursor;
    let start = text[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[cursor..]
        .find('\n')
        .map(|i| cursor + i)
        .unwrap_or(text.len());
    (&text[start..end], cursor - start, start > 0)
}
