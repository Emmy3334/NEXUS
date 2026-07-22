//! `source` / `.` — run a file in the current shell environment.

use super::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};
use std::path::Path;

/// Queue a script file for the REPL to execute with the live environment.
pub(super) fn source(
    argv: &[String],
    _shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let Some(path) = argv.get(1).map(String::as_str) else {
        writeln!(stderr, "source: Too few arguments.")?;
        return Ok(BuiltinResult::Status(1));
    };
    if argv.len() > 2 {
        writeln!(stderr, "source: Too many arguments.")?;
        return Ok(BuiltinResult::Status(1));
    }
    if !Path::new(path).is_file() {
        writeln!(stderr, "{path}: No such file.")?;
        return Ok(BuiltinResult::Status(1));
    }
    Ok(BuiltinResult::Source(path.to_owned()))
}
