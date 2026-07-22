//! `repeat` — run a simple command N times.

use crate::builtins::BuiltinResult;

use std::io::{self, Write};

pub(super) fn repeat_cmd(
    argv: &[String],
    stderr: &mut impl Write,
) -> io::Result<Option<BuiltinResult>> {
    if argv.len() < 3 {
        writeln!(stderr, "repeat: Too few arguments.")?;
        return Ok(Some(BuiltinResult::Status(1)));
    }
    let Ok(count) = argv[1].parse::<u32>() else {
        writeln!(stderr, "repeat: Invalid count.")?;
        return Ok(Some(BuiltinResult::Status(1)));
    };
    Ok(Some(BuiltinResult::Repeat {
        count,
        argv: argv[2..].to_vec(),
    }))
}
