//! `@docker logs`.

use crate::builtins::BuiltinResult;
use crate::heal;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let Some(target) = args.get(1).map(String::as_str) else {
        writeln!(stderr, "usage: @docker logs <name|id>")?;
        return Ok(BuiltinResult::Status(1));
    };
    if args.len() > 2 {
        writeln!(stderr, "usage: @docker logs <name|id>")?;
        return Ok(BuiltinResult::Status(1));
    }
    match heal::write_container_logs(target, stdout, stderr) {
        Ok(()) => Ok(BuiltinResult::Status(0)),
        Err(err) => {
            writeln!(stderr, "@docker: {err}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
