//! Read–eval–print loop (prompt → lex → parse → exec/builtins).
//!
//! Dragon Book pipeline: acquire line → lexical analysis → list/pipeline
//! parse → execute against an owned environment copy. Semicolon lists and
//! `|` pipelines are both executed.

use crate::env::ShellEnvironment;
use crate::exec::{self, CommandResult};
use crate::lex;
use crate::parse;

use std::io::{self, BufRead, Write};

const PROMPT: &str = "$> ";
const SUCCESS_EXIT: u8 = 0;

/// The REPL's own I/O streams, bundled so helper functions don't need one
/// parameter per stream.
struct ReplIo<'a, I, O, E> {
    stdin: &'a mut I,
    stdout: &'a mut O,
    stderr: &'a mut E,
}

/// Outcome of reading and parsing one line.
enum ParseOutcome<'a> {
    /// Stdin closed.
    Eof,
    /// Nothing to run (blank line, or only `;`).
    Blank,
    /// Lex or parse error; already reported to `stderr`.
    Failed(u8),
    /// A command list ready to execute.
    Ready(parse::CommandList<'a>),
}

/// Outcome of one read–eval iteration.
enum StepOutcome {
    /// Keep looping with this status as `last_status`.
    Continue(u8),
    /// Stop the shell (EOF or `exit`) with this status.
    Stop(u8),
}

/// Run the interactive (or piped) read–eval loop.
///
/// - Prints [`PROMPT`] only when `interactive` is true (TTY stdin).
/// - Blank lines re-prompt.
/// - EOF returns the last command’s status (or `0` if none ran).
/// - `exit` ends the shell immediately with the requested status.
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
    let mut line_buffer = String::new();
    let mut tokens = Vec::new();
    let mut argv = Vec::new();
    let mut last_status = SUCCESS_EXIT;

    loop {
        match step(
            &mut io,
            interactive,
            &mut line_buffer,
            &mut tokens,
            &mut argv,
            &mut shell_env,
            last_status,
        )? {
            StepOutcome::Continue(status) => last_status = status,
            StepOutcome::Stop(status) => return Ok(status),
        }
    }
}

/// One prompt → read → parse → execute iteration.
fn step<I: BufRead, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    line_buffer: &mut String,
    tokens: &mut Vec<lex::Token>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<StepOutcome> {
    write_prompt(io.stdout, interactive)?;

    let command_list = match read_and_parse(io, line_buffer, tokens)? {
        ParseOutcome::Eof => return Ok(StepOutcome::Stop(last_status)),
        ParseOutcome::Blank => return Ok(StepOutcome::Continue(last_status)),
        ParseOutcome::Failed(code) => return Ok(StepOutcome::Continue(code)),
        ParseOutcome::Ready(list) => list,
    };

    match run_ready_command(&command_list, io, argv, shell_env, last_status)? {
        CommandResult::Status(code) => Ok(StepOutcome::Continue(code)),
        CommandResult::Exit(code) => Ok(StepOutcome::Stop(code)),
    }
}

/// Read one line, tokenize, and parse it. Borrows `line_buffer` for the
/// lifetime of the returned [`ParseOutcome::Ready`] command list.
fn read_and_parse<'a, I: BufRead, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    line_buffer: &'a mut String,
    tokens: &mut Vec<lex::Token>,
) -> io::Result<ParseOutcome<'a>> {
    line_buffer.clear();
    let bytes_read = io.stdin.read_line(line_buffer)?;
    if bytes_read == 0 {
        return Ok(ParseOutcome::Eof);
    }
    if line_buffer.trim().is_empty() {
        return Ok(ParseOutcome::Blank);
    }

    let command_line = line_buffer.trim_end_matches(['\n', '\r']);
    if let Err(error) = lex::tokenize_into(command_line, tokens) {
        writeln!(io.stderr, "{}", error.message())?;
        return Ok(ParseOutcome::Failed(1));
    }
    debug_assert!(tokens_are_well_formed(command_line, tokens));

    match parse::parse_line(command_line, tokens) {
        Ok(Some(list)) => Ok(ParseOutcome::Ready(list)),
        Ok(None) => Ok(ParseOutcome::Blank),
        Err(error) => {
            writeln!(io.stderr, "{}", error.message())?;
            Ok(ParseOutcome::Failed(1))
        }
    }
}

/// Collect heredoc bodies (if any) then execute the parsed command list.
fn run_ready_command<I: BufRead, O: Write, E: Write>(
    command_list: &parse::CommandList<'_>,
    io: &mut ReplIo<'_, I, O, E>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<CommandResult> {
    let heredoc_bodies = match exec::collect_heredoc_bodies(
        command_list,
        shell_env,
        last_status,
        io.stdin,
        io.stderr,
    )? {
        Ok(bodies) => bodies,
        Err(code) => return Ok(CommandResult::Status(code)),
    };

    exec::execute_list(
        command_list,
        argv,
        shell_env,
        last_status,
        heredoc_bodies,
        io.stdout,
        io.stderr,
    )
}

fn tokens_are_well_formed(source: &str, tokens: &[lex::Token]) -> bool {
    tokens.iter().all(|token| {
        token.start <= token.end
            && token
                .try_lexeme(source)
                .is_some_and(|lexeme| !lexeme.is_empty())
    })
}

fn write_prompt(stdout: &mut impl Write, interactive: bool) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    write!(stdout, "{PROMPT}")?;
    stdout.flush()
}
