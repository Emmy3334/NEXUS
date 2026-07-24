//! Line loop for body replay.

use super::super::line_edit::ReplInput;
use super::super::ReplIo;
use super::step::{self, Replay};
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;

use std::io::{self, Write};

pub(super) fn replay_lines<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let mut line_buffer = String::new();
    let mut expanded = String::new();
    let mut tokens = Vec::new();
    loop {
        match step::replay_one(
            io,
            &mut line_buffer,
            &mut expanded,
            &mut tokens,
            argv,
            shell_env,
            last_status,
        )? {
            Replay::Continue(code) => last_status = code,
            Replay::Eof => return Ok(CommandResult::Status(last_status)),
            Replay::Exit(code) => return Ok(CommandResult::Exit(code)),
            Replay::Stop(code) => return Ok(CommandResult::Status(code)),
        }
    }
}
