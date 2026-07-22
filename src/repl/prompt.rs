//! Prompt strings for primary and continuation lines.

use std::io::{self, Write};

pub(super) const PRIMARY: &str = "$> ";
pub(super) const CONTINUE: &str = "? ";

pub(super) fn write_primary(stdout: &mut impl Write, interactive: bool) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    write!(stdout, "{PRIMARY}")?;
    stdout.flush()
}

pub(super) fn write_continue(stdout: &mut impl Write) -> io::Result<()> {
    write!(stdout, "{CONTINUE}")?;
    stdout.flush()
}
