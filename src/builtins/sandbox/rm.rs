//! `sandbox rm <name…>`.

use crate::builtins::BuiltinResult;
use crate::sandbox;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if args.len() < 2 {
        writeln!(stderr, "usage: sandbox rm <name…>")?;
        return Ok(BuiltinResult::Status(1));
    }
    for name in &args[1..] {
        if let Err(err) = sandbox::remove_named(name) {
            writeln!(stderr, "sandbox: {err}")?;
            return Ok(BuiltinResult::Status(1));
        }
    }
    Ok(BuiltinResult::Status(0))
}
