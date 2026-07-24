//! Read a line, expand history events, lex, and parse.

use super::control_parse;
use super::line_edit::CompleteCtx;
use super::line_edit::{self, ReadOutcome, ReplInput};
use super::ReplIo;
use crate::case_block::CaseHeader;
use crate::env::ShellEnvironment;
use crate::exec::{self, CommandResult};
use crate::foreach::ForEachHeader;
use crate::functions::FunctionHeader;
use crate::history::{self, ExpandOutcome};
use crate::if_block::IfHeader;
use crate::lex;
use crate::parse;
use crate::while_loop::WhileHeader;

use std::io::{self, BufRead, Write};

/// Outcome of reading and parsing one line.
pub(super) enum ParseOutcome<'a> {
    /// Stdin closed.
    Eof,
    /// Nothing to run (blank line, only `;`, or `:p` print-only).
    Blank,
    /// Lex, history, or parse error; already reported to `stderr`.
    Failed(u8),
    /// A command list ready to execute.
    Ready(parse::CommandList<'a>),
    /// `foreach name ( … )` — body collected by the REPL.
    ForEach(ForEachHeader),
    /// `while ( expr )` — body collected by the REPL.
    While(WhileHeader),
    /// `if ( expr ) then` — body collected by the REPL.
    If(IfHeader),
    /// `name() { … }` / `function name { … }`.
    Function(FunctionHeader),
    /// `case word in` — arms collected by the REPL.
    Case(CaseHeader),
}

/// Read one logical line, expand `!` events, tokenize, and parse.
pub(super) fn read_and_parse<'a, I: ReplInput, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    line_buffer: &mut String,
    expanded: &'a mut String,
    tokens: &mut Vec<lex::Token>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<ParseOutcome<'a>> {
    // Tab complete only needs names on a live TTY; skip the snapshot otherwise.
    let tty = interactive && io.stdin.is_terminal();
    let var_names = if tty {
        shell_env.var_names()
    } else {
        Vec::new()
    };
    let path = crate::harden::effective_path(shell_env);
    let cmd_names = if tty {
        line_edit::command_names(shell_env)
    } else {
        Vec::new()
    };
    let complete_ctx = CompleteCtx {
        var_names: &var_names,
        registry: &shell_env.comp_registry,
        path: &path,
        cmd_names: &cmd_names,
        history: Some(&shell_env.history),
    };
    let prompt_ctx = crate::repl::prompt::PromptContext::from_env(shell_env, last_status);
    match line_edit::read_logical_line(
        io.stdin,
        io.stdout,
        interactive,
        &shell_env.history,
        &mut shell_env.key_bindings,
        line_buffer,
        &mut io.input_queue,
        if tty { Some(&complete_ctx) } else { None },
        Some(&prompt_ctx),
    )? {
        ReadOutcome::Eof => return Ok(ParseOutcome::Eof),
        ReadOutcome::Line => {}
    }
    if line_buffer.trim().is_empty() {
        return Ok(ParseOutcome::Blank);
    }
    // Fast path: no history designators → skip quote-aware `!` scan.
    let needs_hist = line_buffer.as_bytes().contains(&b'!') || line_buffer.starts_with('^');
    let outcome = if needs_hist {
        match history::expand_line(line_buffer, &mut shell_env.history, expanded) {
            Ok(o) => o,
            Err(err) => {
                writeln!(io.stderr, "{}", err.message())?;
                return Ok(ParseOutcome::Failed(1));
            }
        }
    } else {
        expanded.clear();
        expanded.push_str(line_buffer);
        history::ExpandOutcome {
            changed: false,
            print_only: false,
        }
    };
    finish_expand(io, expanded, tokens, shell_env, outcome)
}

fn finish_expand<'a, I: BufRead, O: Write, E: Write>(
    io: &mut ReplIo<'_, I, O, E>,
    expanded: &'a mut String,
    tokens: &mut Vec<lex::Token>,
    shell_env: &mut ShellEnvironment,
    outcome: ExpandOutcome,
) -> io::Result<ParseOutcome<'a>> {
    if outcome.changed {
        writeln!(io.stdout, "{expanded}")?;
    }
    if outcome.print_only {
        return Ok(ParseOutcome::Blank);
    }
    // Scripts / `source` / `~/.nexusrc` can suppress recording.
    if !shell_env.suppress_history {
        let limit = shell_env
            .lookup("histsize")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(history::DEFAULT_HISTSIZE);
        shell_env.history.push_limited(expanded.as_str(), limit);
    }
    tokenize_and_parse(expanded, tokens, io.stderr)
}

fn tokenize_and_parse<'a, E: Write>(
    expanded: &'a str,
    tokens: &mut Vec<lex::Token>,
    stderr: &mut E,
) -> io::Result<ParseOutcome<'a>> {
    if let Err(error) = lex::tokenize_into(expanded, tokens) {
        writeln!(stderr, "{}", error.message())?;
        return Ok(ParseOutcome::Failed(1));
    }
    debug_assert!(tokens_are_well_formed(expanded, tokens));
    if let Some(outcome) = control_parse::try_control(expanded, tokens, stderr)? {
        return Ok(outcome);
    }
    match parse::parse_line(expanded, tokens) {
        Ok(Some(list)) => Ok(ParseOutcome::Ready(list)),
        Ok(None) => Ok(ParseOutcome::Blank),
        Err(error) => {
            writeln!(stderr, "{}", error.message())?;
            Ok(ParseOutcome::Failed(1))
        }
    }
}

/// Collect heredoc bodies (if any) then execute the parsed command list.
pub(super) fn run_ready_command<I: BufRead, O: Write, E: Write>(
    command_list: &parse::CommandList<'_>,
    io: &mut ReplIo<'_, I, O, E>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<CommandResult> {
    run_ready(command_list, io, argv, shell_env, last_status, false)
}

/// Like [`run_ready_command`], but capture external stdout into `io.stdout`
/// (needed when that handle is a redirect file / buffer, not process stdout).
pub(super) fn run_ready_command_captured<I: BufRead, O: Write, E: Write>(
    command_list: &parse::CommandList<'_>,
    io: &mut ReplIo<'_, I, O, E>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
) -> io::Result<CommandResult> {
    run_ready(command_list, io, argv, shell_env, last_status, true)
}

fn run_ready<I: BufRead, O: Write, E: Write>(
    command_list: &parse::CommandList<'_>,
    io: &mut ReplIo<'_, I, O, E>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    capture: bool,
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
    if capture {
        exec::execute_list_captured(
            command_list,
            argv,
            shell_env,
            last_status,
            heredoc_bodies,
            io.stdin,
            io.stdout,
            io.stderr,
        )
    } else {
        exec::execute_list(
            command_list,
            argv,
            shell_env,
            last_status,
            heredoc_bodies,
            io.stdin,
            io.stdout,
            io.stderr,
        )
    }
}

fn tokens_are_well_formed(source: &str, tokens: &[lex::Token]) -> bool {
    tokens.iter().all(|token| {
        token.start <= token.end
            && token
                .try_lexeme(source)
                .is_some_and(|lexeme| !lexeme.is_empty())
    })
}
