//! Heredoc (`<<`) body collection and lookup.

use crate::env::ShellEnvironment;
use crate::expand;
use crate::parse::{CommandList, RedirectKind};

use std::io::{self, BufRead, Write};

/// Cursor over pre-collected heredoc bodies (one per `<<`, left-to-right).
///
/// Takes ownership of each body when applied so we do not clone the payload.
pub(in crate::exec) struct HeredocState {
    bodies: Vec<String>,
    index: usize,
}

impl HeredocState {
    pub(in crate::exec) fn new(bodies: Vec<String>) -> Self {
        Self { bodies, index: 0 }
    }

    pub(super) fn take_next(&mut self, stderr: &mut impl Write) -> io::Result<Result<String, u8>> {
        if self.index >= self.bodies.len() {
            writeln!(stderr, "nexus: missing heredoc body")?;
            return Ok(Err(1));
        }
        let body = std::mem::take(&mut self.bodies[self.index]);
        self.index += 1;
        Ok(Ok(body))
    }
}

/// Read heredoc bodies for every `<<` in `list`, left-to-right.
///
/// Each body is the input lines up to (but not including) a line whose content
/// equals the (quote- and `$`-expanded) delimiter. On EOF before the delimiter, the
/// partial body is kept and a warning is written to `stderr`.
///
/// Returns `Ok(Err(code))` when quote expansion of a delimiter fails.
pub fn collect_heredoc_bodies(
    list: &CommandList<'_>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    input: &mut impl BufRead,
    stderr: &mut impl Write,
) -> io::Result<Result<Vec<String>, u8>> {
    let mut bodies = Vec::new();
    let mut line = String::new();

    for pipeline in &list.pipelines {
        for command in &pipeline.commands {
            for redirect in &command.redirects {
                if redirect.kind == RedirectKind::Heredoc {
                    // Heredoc delimiters are not subject to pathname expansion.
                    let delimiter = match expand_heredoc_delimiter(
                        redirect.path,
                        shell_env,
                        last_status,
                        input,
                        stderr,
                    )? {
                        Ok(d) => d,
                        Err(code) => return Ok(Err(code)),
                    };
                    bodies.push(read_heredoc_body(input, &delimiter, &mut line, stderr)?);
                }
            }
        }
    }

    Ok(Ok(bodies))
}

fn expand_heredoc_delimiter(
    raw: &str,
    shell_env: &ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> io::Result<Result<String, u8>> {
    let mut fields = Vec::new();
    let mut capture = |body: &str| {
        crate::exec::capture_command_output(body, shell_env, last_status, stdin, stderr)
    };
    match expand::expand_word_fields_into(raw, shell_env, last_status, &mut fields, &mut capture) {
        Ok(()) => Ok(Ok(fields
            .into_iter()
            .next()
            .unwrap_or_default()
            .into_string())),
        Err(err) => {
            writeln!(stderr, "{}", err.message())?;
            Ok(Err(1))
        }
    }
}

fn read_heredoc_body(
    input: &mut impl BufRead,
    delimiter: &str,
    line: &mut String,
    stderr: &mut impl Write,
) -> io::Result<String> {
    let mut body = String::new();
    loop {
        line.clear();
        let bytes = input.read_line(line)?;
        if bytes == 0 {
            writeln!(
                stderr,
                "nexus: warning: here-document delimited by end-of-file (wanted `{delimiter}`)"
            )?;
            break;
        }
        let without_ending = line.trim_end_matches(['\n', '\r']);
        if without_ending == delimiter {
            break;
        }
        body.push_str(line);
    }
    Ok(body)
}
