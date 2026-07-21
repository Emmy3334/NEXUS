//! Read–eval–print loop (Minishell1: prompt → lex → parse → exec).
//!
//! Dragon Book pipeline: acquire line → lexical analysis → simple parse →
//! external execution. Builtins arrive in a later slice.

use crate::exec;
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
/// - External commands propagate their exit status; not-found → `127`.
pub fn run(
    mut stdin: impl BufRead,
    mut stdout: impl Write,
    mut stderr: impl Write,
    interactive: bool,
) -> io::Result<u8> {
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

        parse::fill_argv(command_line, &tokens, &mut argv);
        if argv.is_empty() {
            continue;
        }

        last_status = exec::execute_external(&argv, &mut stderr)?;
    }
}

fn tokens_are_well_formed(source: &str, tokens: &[lex::Token]) -> bool {
    tokens.iter().all(|token| {
        token.start <= token.end
            && token.end <= source.len()
            && !token.lexeme(source).is_empty()
            && !token.lexeme(source).chars().any(char::is_whitespace)
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
    fn trim_line_ending_strips_crlf() {
        assert_eq!(trim_line_ending("ls\n"), "ls");
        assert_eq!(trim_line_ending("ls\r\n"), "ls");
        assert_eq!(trim_line_ending("ls"), "ls");
    }
}
