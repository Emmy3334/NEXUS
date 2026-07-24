//! `@docker logs`.

use super::args;
use crate::builtins::BuiltinResult;
use crate::heal;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let _follow = args::wants_follow(args);
    let Some(target) = args::first_positional(args, 1) else {
        writeln!(stderr, "usage: @docker logs [-f|--follow] <name|id>")?;
        return Ok(BuiltinResult::Status(1));
    };
    // Follow is accepted for Tab/CLI parity; snapshot logs for now (no stream).
    match heal::write_container_logs(target, stdout, stderr) {
        Ok(()) => Ok(BuiltinResult::Status(0)),
        Err(err) => {
            writeln!(stderr, "@docker: {err}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
