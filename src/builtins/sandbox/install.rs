//! `sandbox install <path> [name]`.

use crate::builtins::BuiltinResult;
use crate::sandbox;

use std::io::{self, Write};
use std::path::Path;

pub(super) fn run(
    args: &[String],
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let Some(path) = args.get(1).map(String::as_str) else {
        writeln!(stderr, "usage: sandbox install <path> [name]")?;
        return Ok(BuiltinResult::Status(1));
    };
    if args.len() > 3 {
        writeln!(stderr, "usage: sandbox install <path> [name]")?;
        return Ok(BuiltinResult::Status(1));
    }
    let name = args.get(2).map(String::as_str);
    match sandbox::install_from_path(Path::new(path), name) {
        Ok(_) => Ok(BuiltinResult::Status(0)),
        Err(err) => {
            writeln!(stderr, "sandbox: {err}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
