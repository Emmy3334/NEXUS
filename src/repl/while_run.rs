//! Expand while condition and execute the loop body.

use super::body_run::run_body;
use super::control_collect::{collect_body, BlockKind};
use super::line_edit::ReplInput;
use super::ReplIo;
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::while_loop::{self, WhileHeader};

use std::io::{self, Write};

/// Collect the body and run it while the condition stays true.
pub(super) fn run_while<I: ReplInput, O: Write, E: Write>(
    header: WhileHeader,
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let Some(body) = collect_body(
        io,
        interactive,
        &shell_env.history,
        &mut shell_env.key_bindings,
        BlockKind::While,
    )?
    else {
        return Ok(CommandResult::Status(1));
    };
    let mut status = last_status;
    loop {
        match while_loop::eval_condition(&header.expr, shell_env, status, io.stdin, io.stderr) {
            Some(true) => {}
            Some(false) => return Ok(CommandResult::Status(status)),
            None => return Ok(CommandResult::Status(1)),
        }
        match run_body(&body, io, shell_env, status, argv)? {
            CommandResult::Status(code) => {
                status = code;
                if shell_env.return_requested() {
                    return Ok(CommandResult::Status(code));
                }
            }
            CommandResult::Exit(code) => return Ok(CommandResult::Exit(code)),
            CommandResult::Source(_) => return Ok(CommandResult::Status(1)),
        }
    }
}
