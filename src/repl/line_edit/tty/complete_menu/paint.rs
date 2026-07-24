//! Paint the completion menu under the input line without moving the prompt.

use super::super::buffer::EditBuffer;
use super::super::draw;
use crate::repl::line_edit::complete;

use std::io::{self, Write};

/// Soft ceiling so a bottom-of-screen prompt never scrolls the viewport away.
const MENU_ROWS_CAP: usize = 12;

/// Paint/rewrite the menu under the input line, then restore the cursor there.
pub(super) fn show(stdout: &mut impl Write, edit: &mut EditBuffer, prompt: &str) -> io::Result<()> {
    let Some(cycle) = edit.complete_cycle.as_ref() else {
        return draw::redraw(stdout, prompt, edit);
    };
    let old_rows = cycle.list_rows;
    let lines = complete::list_menu_fit(&cycle.matches, cycle.highlight, row_budget());
    let new_rows = lines.len();
    if old_rows == 0 {
        write_fresh(stdout, &lines)?;
    } else {
        rewrite(stdout, &lines, old_rows)?;
    }
    if let Some(cycle) = edit.complete_cycle.as_mut() {
        cycle.list_rows = new_rows;
    }
    draw::redraw(stdout, prompt, edit)
}

fn row_budget() -> usize {
    let rows = std::env::var("LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(24usize);
    rows.saturating_sub(2).clamp(3, MENU_ROWS_CAP)
}

fn write_fresh(stdout: &mut impl Write, lines: &[String]) -> io::Result<()> {
    writeln!(stdout)?;
    for line in lines {
        write!(stdout, "\r\x1b[2K{line}\n")?;
    }
    // Cursor sits one row past the menu; climb back to the input row.
    draw::shift_rows(stdout, -((lines.len() + 1) as i32))
}

fn rewrite(stdout: &mut impl Write, lines: &[String], old_rows: usize) -> io::Result<()> {
    let new_rows = lines.len();
    draw::shift_rows(stdout, 1)?;
    for line in lines {
        write!(stdout, "\r\x1b[2K{line}\n")?;
    }
    for _ in new_rows..old_rows {
        write!(stdout, "\r\x1b[2K\n")?;
    }
    let span = old_rows.max(new_rows);
    draw::shift_rows(stdout, -((span + 1) as i32))
}
