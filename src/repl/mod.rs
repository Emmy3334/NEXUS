//! Read–eval–print loop (prompt → history → lex → parse → exec/builtins).
//!
//! Dragon Book pipeline: acquire line → history expansion → lexical analysis →
//! list/pipeline parse → execute against an owned environment copy.

mod line;

use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::lex;

use line::{read_and_parse, run_ready_command, ParseOutcome};
use std::io::{self, BufRead, Write};

const PROMPT: &str = "$> ";
const SUCCESS_EXIT: u8 = 0;

/// The REPL's own I/O streams, bundled so helper functions don't need one
/// parameter per stream.
pub(super) struct ReplIo<'a, I, O, E> {
    pub(super) stdin: &'a mut I,
    pub(super) stdout: &'a mut O,
    pub(super) stderr: &'a mut E,
}

/// Reusable scratch buffers for one REPL session.
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

/// Outcome of one read–eval iteration.
enum StepOutcome {
    /// Keep looping with this status as `last_status`.
    Continue(u8),
    /// Stop the shell (EOF or `exit`) with this status.
    Stop(u8),
}

/// Run the interactive (or piped) read–eval loop.
pub fn run(
    mut stdin: impl BufRead,
    mut stdout: impl Write,
    mut stderr: impl Write,
    interactive: bool,
) -> io::Result<u8> {
    let mut io = ReplIo {
        stdin: &mut stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
    };
    let mut shell_env = ShellEnvironment::capture();
    let mut buffers = ReplBuffers::new();
    let mut last_status = SUCCESS_EXIT;
    if interactive {
        crate::jobs::install_interactive_handlers()?;
    }
    loop {
        match step(
            &mut io,
            interactive,
            &mut buffers,
            &mut shell_env,
            last_status,
        )? {
            StepOutcome::Continue(status) => last_status = status,
            StepOutcome::Stop(status) => return Ok(status),
        }
    }
}

/// One prompt → read → history → parse → execute iteration.
fn step<I: BufRead, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    buffers: &mut ReplBuffers,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<StepOutcome> {
    notify_completed_jobs(io.stderr, shell_env)?;
    write_prompt(io.stdout, interactive)?;
    let command_list = match read_and_parse(
        io,
        &mut buffers.line,
        &mut buffers.expanded,
        &mut buffers.tokens,
        shell_env,
    )? {
        ParseOutcome::Eof => return Ok(StepOutcome::Stop(last_status)),
        ParseOutcome::Blank => return Ok(StepOutcome::Continue(last_status)),
        ParseOutcome::Failed(code) => return Ok(StepOutcome::Continue(code)),
        ParseOutcome::Ready(list) => list,
    };
    match run_ready_command(&command_list, io, &mut buffers.argv, shell_env, last_status)? {
        CommandResult::Status(code) => Ok(StepOutcome::Continue(code)),
        CommandResult::Exit(code) => Ok(StepOutcome::Stop(code)),
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

fn write_prompt(stdout: &mut impl Write, interactive: bool) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    write!(stdout, "{PROMPT}")?;
    stdout.flush()
}
