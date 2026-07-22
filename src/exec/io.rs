//! Shared I/O + stdout mode for one execute / capture run.

use super::stdout_mode::StdoutMode;

use std::io::{BufRead, Write};

/// Stdin/stdout/stderr plus whether externals inherit or pipe stdout.
pub(crate) struct ExecIo<'a, I: BufRead, O: Write, E: Write> {
    pub(crate) stdin: &'a mut I,
    pub(crate) stdout: &'a mut O,
    pub(crate) stderr: &'a mut E,
    pub(crate) stdout_mode: StdoutMode,
}
