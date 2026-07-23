//! `@kube describe`.

use super::{args, print};
use crate::builtins::BuiltinResult;
use crate::kube;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match args.get(1).map(String::as_str) {
        Some("pod" | "pods") => describe_pod(args, stdout, stderr),
        None => {
            writeln!(stderr, "usage: @kube describe pod <name> [-n NS]")?;
            Ok(BuiltinResult::Status(1))
        }
        Some(other) => {
            writeln!(stderr, "@kube: unknown describe resource: {other}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn describe_pod(
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
    let Some(name) = args::first_positional(args, 2) else {
        writeln!(stderr, "usage: @kube describe pod <name> [-n NS]")?;
        return Ok(BuiltinResult::Status(1));
    };
    print::text(kube::describe_pod(name, ns.as_deref()), stdout, stderr)
}
