//! `typeset` — function-local scalars, optional `-x`, and `-a` arrays.

mod array;
mod flags;
mod scalar;

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let (mode, args) = match flags::parse(argv) {
        Ok(v) => v,
        Err(msg) => {
            writeln!(stderr, "typeset: {msg}")?;
            return Ok(BuiltinResult::Status(1));
        }
    };
    match mode {
        flags::Mode::Array => array::run(args, shell_env, stderr),
        flags::Mode::Scalar { export } => {
            if !export && shell_env.func_depth() == 0 {
                writeln!(stderr, "typeset: not in a function")?;
                return Ok(BuiltinResult::Status(1));
            }
            scalar::run(args, export, shell_env, stderr)
        }
    }
}
