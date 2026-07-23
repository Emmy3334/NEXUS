//! `@kube get` aliases.

use super::{pods, print};
use crate::builtins::BuiltinResult;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match args.get(1).map(String::as_str) {
        Some("pods" | "pod") => pods::run(&args[1..], stdout, stderr),
        Some("nodes" | "node") => print::nodes(stdout, stderr),
        None => {
            writeln!(stderr, "usage: @kube get pods|nodes")?;
            Ok(BuiltinResult::Status(1))
        }
        Some(other) => {
            writeln!(stderr, "@kube: unknown get resource: {other}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
