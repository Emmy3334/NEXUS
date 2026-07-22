//! `repeat` execution helper (kept out of `command.rs` size limits).

use super::command::execute_command_mode;
use super::stdout_mode::StdoutMode;
use super::CommandResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run_repeat(
    stdout_mode: StdoutMode,
    count: u32,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    for _ in 0..count {
        match execute_command_mode(stdout_mode, argv, shell_env, last_status, stdout, stderr)? {
            CommandResult::Status(code) => last_status = code,
            other => return Ok(other),
        }
    }
    Ok(CommandResult::Status(last_status))
}
