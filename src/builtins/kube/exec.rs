//! `@kube exec` — thin non-TTY remote command.

use super::args;
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
    let (pod, cmd) = match args::exec_pod_and_cmd(args) {
        Ok(pair) => pair,
        Err(msg) => {
            writeln!(stderr, "{msg}")?;
            return Ok(BuiltinResult::Status(1));
        }
    };
    match kube::pod_exec(pod, ns.as_deref(), &cmd, stdout, stderr) {
        Ok(code) => Ok(BuiltinResult::Status(code)),
        Err(err) => {
            writeln!(stderr, "@kube: {}", kube::format_err(&err))?;
            Ok(BuiltinResult::Status(1))
        }
    }
}
