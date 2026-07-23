//! `return` builtin — leave the current function body.

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if shell_env.func_depth() == 0 {
        writeln!(stderr, "return: not in a function")?;
        return Ok(BuiltinResult::Status(1));
    }
    let code = match argv.get(1) {
        None => last_status,
        Some(text) => match text.parse::<u8>() {
            Ok(n) => n,
            Err(_) => {
                writeln!(stderr, "return: Expression Syntax.")?;
                return Ok(BuiltinResult::Status(1));
            }
        },
    };
    shell_env.request_return(code);
    Ok(BuiltinResult::Status(code))
}
