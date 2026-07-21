//! Read–eval–print loop (Minishell1: prompt + read + lex).
//!
//! Dragon Book pipeline so far: acquire line → lexical analysis.
//! Parsing and execution arrive in later feature slices.

use crate::lex;

use std::io::{self, BufRead, Write};

const PROMPT: &str = "$> ";
const SUCCESS_EXIT: u8 = 0;

/// Run the interactive (or piped) read loop.
///
/// - Prints [`PROMPT`] only when `interactive` is true (TTY stdin).
/// - Blank lines re-prompt.
/// - EOF (Ctrl-D / end of pipe) returns [`SUCCESS_EXIT`].
/// - Non-blank lines are tokenized; execution is not wired yet.
pub fn run(mut stdin: impl BufRead, mut stdout: impl Write, interactive: bool) -> io::Result<u8> {
    // Hot path: reuse one line buffer and one token buffer across iterations.
    let mut line_buffer = String::new();
    let mut tokens = Vec::new();

    loop {
        write_prompt(&mut stdout, interactive)?;

        line_buffer.clear();
        let bytes_read = stdin.read_line(&mut line_buffer)?;
        if bytes_read == 0 {
            return Ok(SUCCESS_EXIT);
        }

        if is_blank_line(&line_buffer) {
            continue;
        }

        let command_line = trim_line_ending(&line_buffer);
        lex::tokenize_into(command_line, &mut tokens);
        // Slice boundary: tokens ready for parse/exec next.
        debug_assert!(tokens_are_well_formed(command_line, &tokens));
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

    fn run_piped(input: &str) -> (u8, Vec<u8>) {
        let mut stdin = Cursor::new(input);
        let mut stdout = Vec::new();
        let code = run(&mut stdin, &mut stdout, false).unwrap();
        (code, stdout)
    }

    #[test]
    fn eof_exits_zero_without_prompt_noise() {
        let (code, stdout) = run_piped("");
        assert_eq!(code, SUCCESS_EXIT);
        assert!(stdout.is_empty());
    }

    #[test]
    fn blank_and_nonblank_lines_then_eof() {
        let (code, stdout) = run_piped("\n   \nls\n");
        assert_eq!(code, SUCCESS_EXIT);
        assert!(stdout.is_empty());
    }

    #[test]
    fn interactive_prints_prompt_before_each_read() {
        let mut stdin = Cursor::new("\nls\n");
        let mut stdout = Vec::new();
        let code = run(&mut stdin, &mut stdout, true).unwrap();
        assert_eq!(code, SUCCESS_EXIT);
        // prompt → blank → prompt → "ls" → prompt → EOF
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
