//! `heal` / `doctor` — heal settings and backend reachability.

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;
use crate::heal;

use std::io::{self, Write};

pub(super) fn heal_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let name = argv.first().map(String::as_str).unwrap_or("heal");
    match argv.get(1).map(String::as_str) {
        None => print_status(shell_env, stdout),
        Some("help" | "-h" | "--help") => {
            writeln!(stderr, "usage: {name}")?;
            Ok(BuiltinResult::Status(1))
        }
        Some(other) => {
            writeln!(stderr, "{name}: unknown subcommand: {other}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn print_status(
    shell_env: &ShellEnvironment,
    stdout: &mut impl Write,
) -> io::Result<BuiltinResult> {
    for line in heal::status_lines(shell_env) {
        writeln!(stdout, "{line}")?;
    }
    Ok(BuiltinResult::Status(0))
}
