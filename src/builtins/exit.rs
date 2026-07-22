//! `exit` builtin — terminate the shell with a status code.

use super::BuiltinResult;

use std::io::{self, Write};

pub(super) fn exit_cmd(
    argv: &[String],
    last_status: u8,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match argv.len() {
        1 => Ok(BuiltinResult::Exit(last_status)),
        2 => match parse_exit_status(&argv[1]) {
            Some(code) => Ok(BuiltinResult::Exit(code)),
            None => {
                writeln!(stderr, "exit: Expression Syntax.")?;
                Ok(BuiltinResult::Status(1))
            }
        },
        _ => {
            writeln!(stderr, "exit: Expression Syntax.")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn parse_exit_status(text: &str) -> Option<u8> {
    text.parse::<i32>().ok().map(|n| n as u8)
}
