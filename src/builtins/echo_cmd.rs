//! `echo` builtin — print arguments (optional `-n` suppresses the trailing newline).

use crate::builtins::BuiltinResult;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    stdout: &mut impl Write,
    _stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let (no_newline, args) = match argv.get(1).map(String::as_str) {
        Some("-n") => (true, &argv[2..]),
        _ => (false, argv.get(1..).unwrap_or(&[])),
    };
    let mut first = true;
    for arg in args {
        if !first {
            write!(stdout, " ")?;
        }
        write!(stdout, "{arg}")?;
        first = false;
    }
    if !no_newline {
        writeln!(stdout)?;
    }
    Ok(BuiltinResult::Status(0))
}
