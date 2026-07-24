//! Replay a list of body lines through the normal read/parse/exec path.

mod replay;
mod step;

use super::history_suppress;
use super::line_edit::ReplInput;
use super::ReplIo;
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;

use std::collections::VecDeque;
use std::io::{self, Cursor, Write};

/// Join `body` lines and replay them (nested control is re-collected from stdin).
pub(crate) fn run_body<I: ReplInput, O: Write, E: Write>(
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
    replay::replay_lines(&mut nested, shell_env, last_status, argv)
}

/// Run a stored function body string (same path as loop/if/case bodies).
pub(crate) fn run_body_str(
    body: &str,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let lines: Vec<String> = body.lines().map(str::to_owned).collect();
    let mut argv = Vec::new();
    history_suppress::with_suppressed(shell_env, |env| {
        let mut stdin = Cursor::new(Vec::<u8>::new());
        let mut io = ReplIo {
            stdin: &mut stdin,
            stdout,
            stderr,
            input_queue: VecDeque::new(),
        };
        run_body(&lines, &mut io, env, last_status, &mut argv)
    })
}
