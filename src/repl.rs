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
    let mut shell_env = ShellEnvironment::capture();
    // Hot path: reuse line, token, and owned-argv buffers across iterations.
    let mut line_buffer = String::new();
    let mut tokens = Vec::new();
    let mut argv = Vec::new();
    let mut last_status = SUCCESS_EXIT;

    loop {
        write_prompt(&mut stdout, interactive)?;

        line_buffer.clear();
        let bytes_read = stdin.read_line(&mut line_buffer)?;
        if bytes_read == 0 {
            return Ok(last_status);
        }

        if is_blank_line(&line_buffer) {
            continue;
        }

        let command_line = trim_line_ending(&line_buffer);
        if let Err(error) = lex::tokenize_into(command_line, &mut tokens) {
            writeln!(stderr, "{}", error.message())?;
            last_status = 1;
            continue;
        }
        debug_assert!(tokens_are_well_formed(command_line, &tokens));

        let command_list = match parse::parse_line(command_line, &tokens) {
            Ok(Some(list)) => list,
            Ok(None) => continue,
            Err(error) => {
                writeln!(stderr, "{}", error.message())?;
                last_status = 1;
                continue;
            }
        };

        let heredoc_bodies =
            match exec::collect_heredoc_bodies(&command_list, &mut stdin, &mut stderr)? {
                Ok(bodies) => bodies,
                Err(code) => {
                    last_status = code;
                    continue;
                }
            };

        match exec::execute_list(
            &command_list,
            &mut argv,
            &mut shell_env,
            last_status,
            heredoc_bodies,
            &mut stdout,
            &mut stderr,
        )? {
            CommandResult::Status(code) => last_status = code,
            CommandResult::Exit(code) => return Ok(code),
        }
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

fn write_prompt(stdout: &mut impl Write, interactive: bool) -> io::Result<()> {
    if !interactive {
        return Ok(());
    }
    write!(stdout, "{PROMPT}")?;
    stdout.flush()
}

fn is_blank_line(line: &str) -> bool {
    line.trim().is_empty()
}

fn trim_line_ending(line: &str) -> &str {
    line.trim_end_matches(['\n', '\r'])
}
