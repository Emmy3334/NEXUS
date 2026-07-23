//! `@kube logs`.

use super::{args, print};
use crate::builtins::BuiltinResult;
use crate::kube;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let ns = match args::namespace_opt(args) {
        Ok(ns) => ns,
        Err(msg) => {
            writeln!(stderr, "@kube: {msg}")?;
            return Ok(BuiltinResult::Status(1));
        }
    };
    let follow = args::wants_follow(args);
    let Some(pod) = args::first_positional(args, 1) else {
        writeln!(stderr, "usage: @kube logs [-n NS] [-f|--follow] <pod>")?;
        return Ok(BuiltinResult::Status(1));
    };
    if follow {
        return match kube::pod_logs_follow(pod, ns.as_deref(), 100, stdout) {
            Ok(()) => Ok(BuiltinResult::Status(0)),
            Err(err) => {
                writeln!(stderr, "@kube: {}", kube::format_err(&err))?;
                Ok(BuiltinResult::Status(1))
            }
        };
    }
    print::text(kube::pod_logs(pod, ns.as_deref(), 100), stdout, stderr)
}
