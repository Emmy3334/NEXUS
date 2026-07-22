//! Read–eval–print loop (prompt → history → lex → parse → exec/builtins).
//!
//! Dragon Book pipeline: acquire line → history expansion → lexical analysis →
//! list/pipeline parse → execute against an owned environment copy.

mod line;
mod line_edit;
mod prompt;
mod script;

pub use line_edit::{Action, HistoryRecall, KeyBindings, ReplInput};
pub use script::run_script;

use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::lex;

use line::{read_and_parse, run_ready_command, ParseOutcome};
use std::io::{self, Write};

const SUCCESS_EXIT: u8 = 0;

/// The REPL's own I/O streams, bundled so helper functions don't need one
/// parameter per stream.
pub(super) struct ReplIo<'a, I, O, E> {
    pub(super) stdin: &'a mut I,
    pub(super) stdout: &'a mut O,
    pub(super) stderr: &'a mut E,
}

struct ReplBuffers {
    line: String,
    expanded: String,
    tokens: Vec<lex::Token>,
    argv: Vec<String>,
}

impl ReplBuffers {
    fn new() -> Self {
        Self {
            line: String::new(),
            expanded: String::new(),
            tokens: Vec::new(),
            argv: Vec::new(),
        }
    }
}

enum StepOutcome {
    Continue(u8),
    /// Stdin closed; keep status but do not treat as `exit`.
    Eof(u8),
    /// `exit` builtin (or nested source that exited).
    Exit(u8),
}

pub(super) enum LoopEnd {
    Status(u8),
    Exit(u8),
}

/// Run the interactive (or piped) read–eval loop with a fresh environment.
pub fn run(
    stdin: impl ReplInput,
    stdout: impl Write,
    stderr: impl Write,
    interactive: bool,
) -> io::Result<u8> {
    let mut shell_env = ShellEnvironment::capture();
    run_with_env(stdin, stdout, stderr, interactive, &mut shell_env)
}

/// Like [`run`], using a caller-owned environment (scripts / tests).
pub fn run_with_env(
    mut stdin: impl ReplInput,
    mut stdout: impl Write,
    mut stderr: impl Write,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
) -> io::Result<u8> {
    let mut io = ReplIo {
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    if interactive {
        crate::jobs::install_interactive_handlers()?;
    }
    Ok(match run_loop(&mut io, interactive, shell_env)? {
        LoopEnd::Status(code) | LoopEnd::Exit(code) => code,
    })
}

pub(super) fn run_loop<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
) -> io::Result<LoopEnd> {
    let mut buffers = ReplBuffers::new();
    let mut last_status = SUCCESS_EXIT;
    loop {
        match step(io, interactive, &mut buffers, shell_env, last_status)? {
            StepOutcome::Continue(status) => last_status = status,
            StepOutcome::Eof(status) => return Ok(LoopEnd::Status(status)),
            StepOutcome::Exit(status) => return Ok(LoopEnd::Exit(status)),
        }
    }
}

fn step<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    buffers: &mut ReplBuffers,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<StepOutcome> {
    notify_completed_jobs(io.stderr, shell_env)?;
    let command_list = match read_and_parse(
        io,
        interactive,
        &mut buffers.line,
        &mut buffers.expanded,
        &mut buffers.tokens,
        shell_env,
    )? {
        ParseOutcome::Eof => return Ok(StepOutcome::Eof(last_status)),
        ParseOutcome::Blank => return Ok(StepOutcome::Continue(last_status)),
        ParseOutcome::Failed(code) => return Ok(StepOutcome::Continue(code)),
        ParseOutcome::Ready(list) => list,
    };
    match run_ready_command(&command_list, io, &mut buffers.argv, shell_env, last_status)? {
        CommandResult::Status(code) => Ok(StepOutcome::Continue(code)),
        CommandResult::Exit(code) => Ok(StepOutcome::Exit(code)),
        CommandResult::Source(path) => match script::source_path(&path, io, shell_env)? {
            LoopEnd::Status(code) => Ok(StepOutcome::Continue(code)),
            LoopEnd::Exit(code) => Ok(StepOutcome::Exit(code)),
        },
    }
}

fn notify_completed_jobs(
    stderr: &mut impl Write,
    shell_env: &mut ShellEnvironment,
) -> io::Result<()> {
    for (id, command, status) in shell_env.jobs.take_notifications() {
        writeln!(stderr, "[{id}]  Done ({status})                 {command}")?;
    }
    Ok(())
}
