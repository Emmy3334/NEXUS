//! Write primary / continuation prompts to the TTY.

use super::{format_primary, format_primary_with, PromptContext, CONTINUE};
use std::io::{self, Write};

pub(crate) fn write_primary(stdout: &mut impl Write, interactive: bool) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    write!(stdout, "{}", format_primary())?;
    stdout.flush()
}

pub(crate) fn write_primary_ctx(
    stdout: &mut impl Write,
    interactive: bool,
    ctx: &PromptContext,
) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    write!(stdout, "{}", format_primary_with(ctx))?;
    stdout.flush()
}

pub(crate) fn write_continue(stdout: &mut impl Write) -> io::Result<()> {
    write!(stdout, "{CONTINUE}")?;
    stdout.flush()
}
