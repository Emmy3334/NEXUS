//! `@kube` — native Kubernetes API.

mod args;
mod describe;
mod exec;
mod get;
mod logs;
mod pods;
mod print;

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn kube_cmd(
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
                "usage: @kube nodes | @kube pods [-n NS|-A] | \
                 @kube logs [-n NS] [-f|--follow] <pod> | \
                 @kube exec [-n NS] <pod> -- <cmd> [args…] | \
                 @kube get pods|nodes | @kube describe pod <name> [-n NS]"
            )?;
            Ok(BuiltinResult::Status(1))
        }
        Some("nodes" | "node") => print::nodes(stdout, stderr),
        Some("pods") => pods::run(args, stdout, stderr),
        Some("logs") => logs::run(args, stdout, stderr),
        Some("exec") => exec::run(args, stdout, stderr),
        Some("get") => get::run(args, stdout, stderr),
        Some("describe") => describe::run(args, stdout, stderr),
        Some(other) => {
            writeln!(stderr, "@kube: unknown subcommand: {other}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
