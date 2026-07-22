//! Builtin name → handler dispatch.

mod bonus;
mod core;

use super::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn dispatch(
    name: &str,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<Option<BuiltinResult>> {
    if let Some(result) = core::run(name, argv, shell_env, last_status, stdout, stderr)? {
        return Ok(Some(result));
    }
    bonus::run(name, argv, shell_env, last_status, stdout, stderr)
}
