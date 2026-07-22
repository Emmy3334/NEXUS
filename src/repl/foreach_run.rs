//! Expand foreach items and execute the loop body.

use super::control_collect::{collect_body, BlockKind};
use super::line;
use super::line_edit::ReplInput;
use super::while_run;
use super::{script, LoopEnd, ReplIo};
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::foreach::ForEachHeader;
use crate::lex;
use crate::parse;

use std::collections::VecDeque;
use std::io::{self, Cursor, Write};

/// Collect the body, expand the word list, and run each iteration.
pub(super) fn run_foreach<I: ReplInput, O: Write, E: Write>(
    header: ForEachHeader,
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let Some(body) = collect_body(io, interactive, &shell_env.history, BlockKind::ForEach)? else {
        return Ok(CommandResult::Status(1));
    };
    let items = match expand_items(&header.items, shell_env, last_status, io)? {
        Some(items) => items,
        None => return Ok(CommandResult::Status(1)),
    };
    let mut status = last_status;
    for item in items {
        shell_env.set_local(&header.var, item);
        match run_body(&body, io, shell_env, status, argv)? {
            CommandResult::Status(code) => status = code,
            CommandResult::Exit(code) => return Ok(CommandResult::Exit(code)),
            CommandResult::Source(_) => return Ok(CommandResult::Status(1)),
        }
    }
    Ok(CommandResult::Status(status))
}

fn expand_items<I: ReplInput, O: Write, E: Write>(
    raw: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ReplIo<'_, I, O, E>,
) -> io::Result<Option<Vec<String>>> {
    let mut out = Vec::new();
    let words: Vec<&str> = raw.iter().map(String::as_str).collect();
    if let Err(err) = parse::fill_argv(
        &words,
        &mut out,
        shell_env,
        last_status,
        io.stdin,
        io.stderr,
    ) {
        writeln!(io.stderr, "{}", err.message())?;
        return Ok(None);
    }
    Ok(Some(out))
}

/// Replay body lines through the normal read/parse/exec path.
pub(super) fn run_body<I: ReplInput, O: Write, E: Write>(
    body: &[String],
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let script = if body.is_empty() {
        String::new()
    } else {
        format!("{}\n", body.join("\n"))
    };
    let mut cursor = Cursor::new(script);
    let mut nested = ReplIo {
        stdin: &mut cursor,
        stdout: io.stdout,
        stderr: io.stderr,
        input_queue: VecDeque::new(),
    };
    replay_lines(&mut nested, shell_env, last_status, argv)
}

fn replay_lines<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let mut line_buffer = String::new();
    let mut expanded = String::new();
    let mut tokens = Vec::new();
    loop {
        match replay_one(
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
        }
    }
}

enum Replay {
    Continue(u8),
    Eof,
    Exit(u8),
}

fn replay_one<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    line_buffer: &mut String,
    expanded: &mut String,
    tokens: &mut Vec<lex::Token>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<Replay> {
    match line::read_and_parse(io, false, line_buffer, expanded, tokens, shell_env)? {
        line::ParseOutcome::Eof => Ok(Replay::Eof),
        line::ParseOutcome::Blank => Ok(Replay::Continue(last_status)),
        line::ParseOutcome::Failed(code) => Ok(Replay::Continue(code)),
        line::ParseOutcome::Ready(list) => apply_result(
            line::run_ready_command(&list, io, argv, shell_env, last_status)?,
            io,
            shell_env,
        ),
        line::ParseOutcome::ForEach(header) => apply_result(
            run_foreach(header, io, false, shell_env, last_status, argv)?,
            io,
            shell_env,
        ),
        line::ParseOutcome::While(header) => apply_result(
            while_run::run_while(header, io, false, shell_env, last_status, argv)?,
            io,
            shell_env,
        ),
    }
}

fn apply_result<I: ReplInput, O: Write, E: Write>(
    result: CommandResult,
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
) -> io::Result<Replay> {
    match result {
        CommandResult::Status(code) => Ok(Replay::Continue(code)),
        CommandResult::Exit(code) => Ok(Replay::Exit(code)),
        CommandResult::Source(path) => match script::source_path(&path, io, shell_env)? {
            LoopEnd::Status(code) => Ok(Replay::Continue(code)),
            LoopEnd::Exit(code) => Ok(Replay::Exit(code)),
        },
    }
}
