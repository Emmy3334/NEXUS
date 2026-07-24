//! `@docker ps`.

use super::args;
use crate::builtins::BuiltinResult;
use crate::heal;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let _all = args::wants_all(args);
    if args::first_positional(args, 1).is_some() {
        writeln!(stderr, "usage: @docker ps [-a|--all]")?;
        return Ok(BuiltinResult::Status(1));
    }
    // `-a` accepted for Tab/CLI parity; listing stays running-only for now.
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
