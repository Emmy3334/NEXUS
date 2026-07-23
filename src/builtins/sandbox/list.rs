//! `sandbox list`.

use crate::builtins::BuiltinResult;
use crate::sandbox;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if args.len() > 1 {
        writeln!(stderr, "usage: sandbox list")?;
        return Ok(BuiltinResult::Status(1));
    }
    for name in sandbox::list_names() {
        writeln!(stdout, "{name}")?;
    }
    Ok(BuiltinResult::Status(0))
}
