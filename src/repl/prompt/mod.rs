//! Prompt strings for primary and continuation lines.

mod git;
mod line;

pub(crate) use line::PromptLine;

use std::io::{self, Write};

/// Continuation prompt (multi-line / open quotes).
pub(super) const CONTINUE: &str = "? ";

/// Bare primary prompt when not inside a git work tree.
pub(super) const PRIMARY_BARE: &str = "$> ";

/// Build the interactive primary prompt (`$> ` or `$> [branch*] `).
#[must_use]
pub fn format_primary() -> String {
    match git::segment() {
        Some(seg) => format!("$> [{seg}] "),
        None => PRIMARY_BARE.to_owned(),
    }
}

/// True when `prompt` is a primary line (history recall enabled).
#[must_use]
pub(super) fn is_primary(prompt: &str) -> bool {
    prompt != CONTINUE
}

pub(super) fn write_primary(stdout: &mut impl Write, interactive: bool) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    write!(stdout, "{}", format_primary())?;
    stdout.flush()
}

pub(super) fn write_continue(stdout: &mut impl Write) -> io::Result<()> {
    write!(stdout, "{CONTINUE}")?;
    stdout.flush()
}
