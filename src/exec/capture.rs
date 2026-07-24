//! Capture stdout from a command-substitution body.

use super::cwd;
use crate::env::ShellEnvironment;
use crate::lex::{self, LexError};
use crate::parse::{self, CommandList};

use std::io::{BufRead, Write};

/// Run `source` as a mini command line and return its stdout text.
///
/// Trailing newlines are stripped (tcsh). Uses a cloned environment so
/// builtins inside `` `…` `` / `$(…)` do not mutate the parent shell.
/// Process cwd is restored afterward (like `(…)`), so `cd` does not leak.
/// Nested `<<` bodies are read from `stdin` (same stream as the parent).
pub(crate) fn capture_command_output(
    source: &str,
    shell_env: &ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> Result<String, LexError> {
    let saved_cwd = cwd::save();
    let result = capture_inner(source, shell_env, last_status, stdin, stderr);
    cwd::restore(saved_cwd, stderr);
    result
}

fn capture_inner(
    source: &str,
    shell_env: &ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> Result<String, LexError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let Some(list) = parse_body(trimmed)? else {
        return Ok(String::new());
    };
    let heredocs = collect_bodies(&list, shell_env, last_status, stdin, stderr)?;
    let mut sub_env = shell_env.clone_for_capture();
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let _ = crate::exec::execute_list_captured(
        &list,
        &mut argv,
        &mut sub_env,
        last_status,
        heredocs,
        stdin,
        &mut stdout,
        stderr,
    )
    .map_err(|_| LexError::CommandSubstitution)?;
    Ok(trim_trailing_newlines(
        String::from_utf8_lossy(&stdout).into_owned(),
    ))
}

fn parse_body(source: &str) -> Result<Option<CommandList<'_>>, LexError> {
    let mut tokens = Vec::new();
    lex::tokenize_into(source, &mut tokens)?;
    match parse::parse_line(source, &tokens) {
        Ok(list) => Ok(list),
        Err(_) => Err(LexError::CommandSubstitution),
    }
}

fn collect_bodies(
    list: &CommandList<'_>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> Result<Vec<String>, LexError> {
    let mut env = shell_env.clone_for_capture();
    match crate::exec::collect_heredoc_bodies(list, &mut env, last_status, stdin, stderr) {
        Ok(Ok(bodies)) => Ok(bodies),
        Ok(Err(_)) | Err(_) => Err(LexError::CommandSubstitution),
    }
}

fn trim_trailing_newlines(mut s: String) -> String {
    while s.ends_with('\n') || s.ends_with('\r') {
        s.pop();
    }
    s
}
