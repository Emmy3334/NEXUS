//! Read–eval–print loop (prompt → lex → parse → exec/builtins).
//!
//! Dragon Book pipeline: acquire line → lexical analysis → list/pipeline
//! parse → execute against an owned environment copy. Semicolon lists run
//! in order; pipe execution lands in a later Minishell2 slice.

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
        lex::tokenize_into(command_line, &mut tokens);
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

        match exec::execute_list(
            &command_list,
            &mut argv,
            &mut shell_env,
            last_status,
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
            && token.try_lexeme(source).is_some_and(|lexeme| {
                !lexeme.is_empty() && !lexeme.chars().any(char::is_whitespace)
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn run_piped(input: &str) -> (u8, Vec<u8>, Vec<u8>) {
        let mut stdin = Cursor::new(input);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
        (code, stdout, stderr)
    }

    #[test]
    fn eof_exits_zero_without_prompt_noise() {
        let (code, stdout, stderr) = run_piped("");
        assert_eq!(code, SUCCESS_EXIT);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn blank_lines_then_true_exits_zero() {
        let (code, stdout, stderr) = run_piped("\n   \ntrue\n");
        assert_eq!(code, 0);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn false_propagates_exit_status() {
        let (code, _, stderr) = run_piped("false\n");
        assert_eq!(code, 1);
        assert!(stderr.is_empty());
    }

    #[test]
    fn missing_command_writes_stderr_and_returns_127() {
        let (code, _, stderr) = run_piped("nexus_no_such_command_42\n");
        assert_eq!(code, 127);
        let message = String::from_utf8(stderr).unwrap();
        assert!(message.contains("Command not found"));
    }

    #[test]
    fn last_status_wins_across_commands() {
        let (code, _, _) = run_piped("true\nfalse\n");
        assert_eq!(code, 1);
        let (code, _, _) = run_piped("false\ntrue\n");
        assert_eq!(code, 0);
    }

    #[test]
    fn exit_builtin_ends_repl() {
        let (code, _, stderr) = run_piped("false\nexit\ntrue\n");
        assert_eq!(code, 1);
        assert!(stderr.is_empty());
    }

    #[test]
    fn exit_with_explicit_status() {
        let (code, _, _) = run_piped("exit 42\n");
        assert_eq!(code, 42);
    }

    #[test]
    fn interactive_prints_prompt_before_each_read() {
        let mut stdin = Cursor::new("\ntrue\n");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = run(&mut stdin, &mut stdout, &mut stderr, true).unwrap();
        assert_eq!(code, 0);
        let out = String::from_utf8(stdout).unwrap();
        assert_eq!(out.matches(PROMPT).count(), 3);
    }

    #[test]
    fn semicolon_lists_execute_in_order() {
        let (code, _, stderr) = run_piped("false ; true\n");
        assert_eq!(code, 0);
        assert!(stderr.is_empty());

        let (code, _, stderr) = run_piped("true ; false\n");
        assert_eq!(code, 1);
        assert!(stderr.is_empty());
    }

    #[test]
    fn pipes_are_not_executed_yet() {
        let (code, _, stderr) = run_piped("true | false\n");
        assert_eq!(code, 1);
        let message = String::from_utf8(stderr).unwrap();
        assert!(message.contains("pipes are not executed yet"));
    }

    #[test]
    fn exit_in_semicolon_list_ends_shell() {
        let (code, _, stderr) = run_piped("exit 9 ; false\n");
        assert_eq!(code, 9);
        assert!(stderr.is_empty());
    }

    #[test]
    fn parse_errors_go_to_stderr() {
        let (code, _, stderr) = run_piped("| true\n");
        assert_eq!(code, 1);
        assert!(String::from_utf8(stderr)
            .unwrap()
            .contains("Invalid null command"));

        let (code, _, stderr) = run_piped("true > out\n");
        assert_eq!(code, 1);
        assert!(String::from_utf8(stderr)
            .unwrap()
            .contains("redirections are not implemented"));
    }

    #[test]
    fn trailing_semicolon_still_runs_simple_command() {
        let (code, _, stderr) = run_piped("true;\n");
        assert_eq!(code, 0);
        assert!(stderr.is_empty());
    }

    #[test]
    fn trim_line_ending_strips_crlf() {
        assert_eq!(trim_line_ending("ls\n"), "ls");
        assert_eq!(trim_line_ending("ls\r\n"), "ls");
        assert_eq!(trim_line_ending("ls"), "ls");
    }
}
