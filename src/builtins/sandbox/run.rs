//! `sandbox` run / soft-host branch (unchanged behavior).

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;
use crate::sandbox;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let target = argv[1].as_str();
    let args = &argv[2..];
    if sandbox::resolve_named(target).is_some() || sandbox::is_wasm_path(target) {
        let code = sandbox::run_named_or_path(target, args, stdout, stderr)?;
        return Ok(BuiltinResult::Status(code));
    }
    let code = sandbox::run_host(&argv[1..], shell_env, stdout, stderr)?;
    Ok(BuiltinResult::Status(code))
}
