//! Core Minishell / early-42sh builtin handlers.

use super::super::{
    alias, at, bindkey, cd, env, exit, history, jobs, set, setenv, source, unalias, unset,
    unsetenv, BuiltinResult,
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
        "cd" => BuiltinResult::Status(cd::cd(argv, shell_env, last_status, stdout, stderr)?),
        "setenv" => BuiltinResult::Status(setenv::setenv(argv, shell_env, stdout, stderr)?),
        "unsetenv" => BuiltinResult::Status(unsetenv::unsetenv(argv, shell_env, stderr)?),
        "env" => BuiltinResult::Status(env::env_cmd(argv, shell_env, stdout, stderr)?),
        "set" => BuiltinResult::Status(set::set(argv, shell_env, stdout, stderr)?),
        "unset" => BuiltinResult::Status(unset::unset(argv, shell_env, stderr)?),
        "alias" => BuiltinResult::Status(alias::alias(argv, shell_env, stdout, stderr)?),
        "unalias" => BuiltinResult::Status(unalias::unalias(argv, shell_env, stderr)?),
        "bindkey" => BuiltinResult::Status(bindkey::bindkey(argv, shell_env, stdout, stderr)?),
        "history" => BuiltinResult::Status(history::history_cmd(argv, shell_env, stdout, stderr)?),
        "jobs" => BuiltinResult::Status(jobs::jobs_cmd(argv, shell_env, stdout, stderr)?),
        "fg" => BuiltinResult::Status(jobs::fg_cmd(argv, shell_env, stderr)?),
        "bg" => BuiltinResult::Status(jobs::bg_cmd(argv, shell_env, stderr)?),
        "source" | "." => source::source(argv, shell_env, stderr)?,
        "@" => BuiltinResult::Status(at::at_cmd(argv, shell_env, stderr)?),
        "exit" => exit::exit_cmd(argv, last_status, stderr)?,
        _ => return Ok(None),
    }))
}
