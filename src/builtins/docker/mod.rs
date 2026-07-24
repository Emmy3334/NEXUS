//! `@docker` — native Docker Engine API (bollard).

mod args;
mod logs;
mod ps;

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn docker_cmd(
    argv: &[String],
    _shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let args = &argv[1..];
    match args.first().map(String::as_str) {
        None | Some("help" | "-h" | "--help") => {
            writeln!(
                stderr,
                "usage: @docker ps [-a|--all] [-q|--quiet] | @docker logs [-f|--follow] <name|id>"
            )?;
            Ok(BuiltinResult::Status(1))
        }
        Some("ps") => ps::run(args, stdout, stderr),
        Some("logs") => logs::run(args, stdout, stderr),
        Some(other) => {
            writeln!(stderr, "@docker: unknown subcommand: {other}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
