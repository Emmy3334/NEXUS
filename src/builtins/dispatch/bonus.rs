//! Bonus builtin handlers (`which`, `repeat`, dir stack, `sandbox`, `@kube`, …).

use super::super::{
    dirstack, docker, heal, kube, local_cmd, repeat, return_cmd, sandbox, which, BuiltinResult,
};
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
        "sandbox" => sandbox::sandbox_cmd(argv, shell_env, stdout, stderr)?,
        "heal" | "doctor" => heal::heal_cmd(argv, shell_env, stdout, stderr)?,
        "@kube" => kube::kube_cmd(argv, shell_env, stdout, stderr)?,
        "@docker" => docker::docker_cmd(argv, shell_env, stdout, stderr)?,
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
        "dirs" => BuiltinResult::Status(dirstack::dirs_cmd(
            argv,
            shell_env,
            last_status,
            stdout,
            stderr,
        )?),
        "return" => return_cmd::run(argv, shell_env, last_status, stdout, stderr)?,
        "local" => local_cmd::run(argv, shell_env, stdout, stderr)?,
        _ => return Ok(None),
    }))
}
