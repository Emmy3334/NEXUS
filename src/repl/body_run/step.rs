//! One body-line step: parse outcome → exec / nested control.

use super::super::case_run;
use super::super::foreach_run;
use super::super::function_run;
use super::super::if_run;
use super::super::line;
use super::super::line_edit::ReplInput;
use super::super::script;
use super::super::while_run;
use super::super::{LoopEnd, ReplIo};
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::lex;

use std::io::{self, Write};

pub(super) enum Replay {
    Continue(u8),
    Eof,
    Exit(u8),
    /// Stop body replay (`return` inside a function); keep `pending_return`.
    Stop(u8),
}

pub(super) fn replay_one<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    line_buffer: &mut String,
    expanded: &mut String,
    tokens: &mut Vec<lex::Token>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<Replay> {
    match line::read_and_parse(
        io,
        false,
        line_buffer,
        expanded,
        tokens,
        shell_env,
        last_status,
    )? {
        line::ParseOutcome::Eof => Ok(Replay::Eof),
        line::ParseOutcome::Blank => Ok(Replay::Continue(last_status)),
        line::ParseOutcome::Failed(code) => Ok(Replay::Continue(code)),
        line::ParseOutcome::Ready(list) => apply_result(
            line::run_ready_command_captured(&list, io, argv, shell_env, last_status)?,
            io,
            shell_env,
        ),
        other => dispatch_control(other, io, argv, shell_env, last_status),
    }
}

fn dispatch_control<I: ReplInput, O: Write, E: Write>(
    outcome: line::ParseOutcome<'_>,
    io: &mut ReplIo<'_, I, O, E>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<Replay> {
    let result = match outcome {
        line::ParseOutcome::ForEach(header) => {
            foreach_run::run_foreach(header, io, false, shell_env, last_status, argv)?
        }
        line::ParseOutcome::While(header) => {
            while_run::run_while(header, io, false, shell_env, last_status, argv)?
        }
        line::ParseOutcome::If(header) => {
            if_run::run_if(header, io, false, shell_env, last_status, argv)?
        }
        line::ParseOutcome::Function(header) => {
            function_run::run_define(header, io, false, shell_env)?
        }
        line::ParseOutcome::Case(header) => {
            case_run::run_case(header, io, false, shell_env, last_status, argv)?
        }
        _ => return Ok(Replay::Continue(last_status)),
    };
    apply_result(result, io, shell_env)
}

fn apply_result<I: ReplInput, O: Write, E: Write>(
    result: CommandResult,
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
) -> io::Result<Replay> {
    match result {
        CommandResult::Status(code) => {
            if shell_env.return_requested() {
                return Ok(Replay::Stop(code));
            }
            Ok(Replay::Continue(code))
        }
        CommandResult::Exit(code) => Ok(Replay::Exit(code)),
        CommandResult::Source(path) => match script::source_path(&path, io, shell_env)? {
            LoopEnd::Status(code) => Ok(Replay::Continue(code)),
            LoopEnd::Exit(code) => Ok(Replay::Exit(code)),
        },
    }
}
