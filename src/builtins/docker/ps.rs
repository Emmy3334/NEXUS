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
    let all = args::wants_all(args);
    let quiet = args::wants_quiet(args);
    if args::first_positional(args, 1).is_some() {
        writeln!(stderr, "usage: @docker ps [-a|--all] [-q|--quiet]")?;
        return Ok(BuiltinResult::Status(1));
    }
    match heal::list_ps_lines(all, quiet) {
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
