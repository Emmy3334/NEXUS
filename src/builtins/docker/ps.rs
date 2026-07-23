//! `@docker ps`.

use crate::builtins::BuiltinResult;
use crate::heal;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if args.len() > 1 {
        writeln!(stderr, "usage: @docker ps")?;
        return Ok(BuiltinResult::Status(1));
    }
    match heal::list_ps_lines() {
        Ok(lines) => {
            for line in lines {
                writeln!(stdout, "{line}")?;
            }
            Ok(BuiltinResult::Status(0))
        }
        Err(err) => {
            writeln!(stderr, "@docker: {err}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
