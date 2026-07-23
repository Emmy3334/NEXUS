//! `sandbox` — Wasm cache + Wasmtime runner + soft host isolation.

mod install;
mod list;
mod rm;
mod run;

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn sandbox_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let args = &argv[1..];
    match args.first().map(String::as_str) {
        None | Some("help" | "-h" | "--help") => {
            writeln!(
                stderr,
                "usage: sandbox <name|path.wasm> [args…] | sandbox install <path> [name] | \
                 sandbox list | sandbox rm <name…>"
            )?;
            Ok(BuiltinResult::Status(1))
        }
        Some("install") => install::run(args, stdout, stderr),
        Some("list") => list::run(args, stdout, stderr),
        Some("rm") => rm::run(args, stdout, stderr),
        Some(_) => run::run(argv, shell_env, stdout, stderr),
    }
}
