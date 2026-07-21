//! Read–eval–print loop (Minishell1 slice 1: prompt + read only).
//!
//! Dragon Book framing: this driver will later feed lines into
//! lexical analysis → parsing → execution. For now it only acquires input.

use std::io::{self, BufRead, Write};

const PROMPT: &str = "$> ";
const SUCCESS_EXIT: u8 = 0;

/// Run the interactive (or piped) read loop.
///
/// - Prints [`PROMPT`] only when `interactive` is true (TTY stdin).
/// - Blank lines re-prompt.
/// - EOF (Ctrl-D / end of pipe) returns [`SUCCESS_EXIT`].
/// - Non-blank lines are accepted and discarded until a later feature wires
///   lex → parse → exec.
pub fn run(mut stdin: impl BufRead, mut stdout: impl Write, interactive: bool) -> io::Result<u8> {
    // Reuse one buffer across iterations (hot path: avoid per-line alloc).
    let mut line_buffer = String::new();

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

        // Slice 1: line acquired; execution lands in the next feature.
        let _command_line = trim_line_ending(&line_buffer);
    }
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
