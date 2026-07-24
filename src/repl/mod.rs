//! Read–eval–print loop (prompt → history → lex → parse → exec/builtins).
//!
//! Dragon Book pipeline: acquire line → history expansion → lexical analysis →
//! list/pipeline parse → execute against an owned environment copy.

mod case_collect;
mod case_run;
mod control_collect;
mod control_parse;
mod foreach_run;
mod function_run;
mod history_persist;
mod history_suppress;
mod if_collect;
mod if_run;
mod line;
mod line_edit;
mod prompt;
mod rc;
mod script;
mod step_prep;
mod tty_session;
mod while_run;

pub use history_persist::{load_session_history, save_session_history};
#[cfg(unix)]
pub use line_edit::{after_line_down, after_line_up, take_complete_line};
pub use line_edit::{
    complete, complete_or_cycle, format_columns, list_display_lines, list_display_lines_width,
    Action, CompleteCycle, HistoryISearch, HistoryRecall, KeyBindings, ReplInput,
};
pub use prompt::format_primary;
pub use rc::{load_startup_rc, source_rc, RcLoad};
pub use script::run_script;

use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::lex;

use line::{read_and_parse, run_ready_command, ParseOutcome};
use std::collections::VecDeque;
use std::io::{self, Write};

/// The REPL's own I/O streams, bundled so helper functions don't need one
/// parameter per stream. `input_queue` holds leftover TTY paste bytes.
pub(super) struct ReplIo<'a, I, O, E> {
    pub(super) stdin: &'a mut I,
    pub(super) stdout: &'a mut O,
    pub(super) stderr: &'a mut E,
    pub(super) input_queue: VecDeque<u8>,
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
        input_queue: VecDeque::new(),
    };
    if interactive {
        crate::jobs::install_interactive_handlers()?;
    }
    let tty = interactive && io.stdin.is_terminal();
    let last_status = match tty_session::boot(tty, shell_env, io.stdout, io.stderr)? {
        tty_session::Boot::Exit(code) => return Ok(code),
        tty_session::Boot::Ready(code) => code,
    };
    let code = match run_loop(&mut io, interactive, shell_env, last_status)? {
        LoopEnd::Status(code) | LoopEnd::Exit(code) => code,
    };
    if tty {
        history_persist::save_session_history(shell_env, io.stderr)?;
    }
    Ok(code)
}

pub(super) fn run_loop<I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
) -> io::Result<LoopEnd> {
    let mut buffers = ReplBuffers::new();
    let mut eof_streak = 0_u32;
    loop {
        match step(
            io,
            interactive,
            &mut buffers,
            shell_env,
            last_status,
            &mut eof_streak,
        )? {
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
    eof_streak: &mut u32,
) -> io::Result<StepOutcome> {
    step_prep::notify_completed_jobs(io.stderr, shell_env)?;
    step_prep::run_precmd_if_interactive(interactive, shell_env, last_status, io)?;
    let parsed = read_and_parse(
        io,
        interactive,
        &mut buffers.line,
        &mut buffers.expanded,
        &mut buffers.tokens,
        shell_env,
    )?;
    let is_eof = matches!(parsed, ParseOutcome::Eof);
    if step_prep::suppress_eof(interactive, is_eof, shell_env, eof_streak, io.stderr)? {
        return Ok(StepOutcome::Continue(last_status));
    }
    execute_parsed(
        parsed,
        io,
        interactive,
        &mut buffers.argv,
        shell_env,
        last_status,
    )
}

fn execute_parsed<I: ReplInput, O: Write, E: Write>(
    parsed: ParseOutcome<'_>,
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<StepOutcome> {
    match parsed {
        ParseOutcome::Eof => Ok(StepOutcome::Eof(last_status)),
        ParseOutcome::Blank => Ok(StepOutcome::Continue(last_status)),
        ParseOutcome::Failed(code) => Ok(StepOutcome::Continue(code)),
        ParseOutcome::Ready(list) => finish_result(
            run_ready_command(&list, io, argv, shell_env, last_status)?,
            io,
            shell_env,
        ),
        ParseOutcome::ForEach(header) => finish_result(
            foreach_run::run_foreach(header, io, interactive, shell_env, last_status, argv)?,
            io,
            shell_env,
        ),
        ParseOutcome::While(header) => finish_result(
            while_run::run_while(header, io, interactive, shell_env, last_status, argv)?,
            io,
            shell_env,
        ),
        ParseOutcome::If(header) => finish_result(
            if_run::run_if(header, io, interactive, shell_env, last_status, argv)?,
            io,
            shell_env,
        ),
        ParseOutcome::Function(header) => finish_result(
            function_run::run_define(header, io, interactive, shell_env)?,
            io,
            shell_env,
        ),
        ParseOutcome::Case(header) => finish_result(
            case_run::run_case(header, io, interactive, shell_env, last_status, argv)?,
            io,
            shell_env,
        ),
    }
}

fn finish_result<I: ReplInput, O: Write, E: Write>(
    result: CommandResult,
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
) -> io::Result<StepOutcome> {
    match result {
        CommandResult::Status(code) => {
            if shell_env.func_depth() > 0 && shell_env.take_return().is_some() {
                // End nested `run_with_env` (function body) without exiting the shell.
                return Ok(StepOutcome::Eof(code));
            }
            Ok(StepOutcome::Continue(code))
        }
        CommandResult::Exit(code) => Ok(StepOutcome::Exit(code)),
        CommandResult::Source(path) => match script::source_path(&path, io, shell_env)? {
            LoopEnd::Status(code) => Ok(StepOutcome::Continue(code)),
            LoopEnd::Exit(code) => Ok(StepOutcome::Exit(code)),
        },
    }
}
