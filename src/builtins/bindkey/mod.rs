//! `bindkey` builtin — list / set / clear interactive editor bindings.

mod args;
mod dispatch;
mod set;
mod usage;

use crate::env::ShellEnvironment;
use args::parse_invocation;

use std::io::{self, Write};

pub(super) fn bindkey(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match parse_invocation(argv) {
        Err(message) => {
            writeln!(stderr, "bindkey: {message}")?;
            Ok(1)
        }
        Ok(inv) => dispatch::run(inv, shell_env, stdout, stderr),
    }
}
