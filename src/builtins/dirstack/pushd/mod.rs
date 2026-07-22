//! `pushd` — push directory, rotate stack, and cd.

mod action;

use super::args::parse_cmd_args;
use super::print::print_stack;
use crate::env::ShellEnvironment;
use action::dispatch;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

pub(crate) use action::push_path;

pub(crate) fn pushd_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let cwd = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    shell_env.dir_stack.ensure_seeded(cwd);
    let (flags, rest) = match parse_cmd_args("pushd", argv, stderr)? {
        Ok(pair) => pair,
        Err(code) => return Ok(code),
    };
    let code = dispatch(rest, shell_env, last_status, stdout, stderr)?;
    if code != 0 {
        return Ok(code);
    }
    print_stack(shell_env, flags, stdout)
}
