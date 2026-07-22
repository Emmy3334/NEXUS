//! Bonus builtin handlers (`which`, `repeat`, dir stack, …).

use super::super::{dirstack, repeat, which, BuiltinResult};
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    name: &str,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<Option<BuiltinResult>> {
    Ok(Some(match name {
        "which" => BuiltinResult::Status(which::which_cmd(argv, shell_env, stdout, stderr)?),
        "where" => BuiltinResult::Status(which::where_cmd(argv, shell_env, stdout, stderr)?),
        "repeat" => return repeat::repeat_cmd(argv, stderr),
        "pushd" => BuiltinResult::Status(dirstack::pushd_cmd(
            argv,
            shell_env,
            last_status,
            stdout,
            stderr,
        )?),
        "popd" => BuiltinResult::Status(dirstack::popd_cmd(
            argv,
            shell_env,
            last_status,
            stdout,
            stderr,
        )?),
        "dirs" => BuiltinResult::Status(dirstack::dirs_cmd(argv, shell_env, stdout, stderr)?),
        _ => return Ok(None),
    }))
}
