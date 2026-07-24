//! Bash/zsh-style `[[ … ]]` conditional command.

mod binary;
mod expr;
mod unary;

use crate::env::ShellEnvironment;
use crate::expand;
use crate::lex::{self, TokenKind};
use crate::parse::SimpleCommand;

use std::io::{self, Write};

/// If `simple` is a lone `[[ … ]]` span, evaluate it; otherwise return `None`.
pub(super) fn try_run(
    simple: &SimpleCommand<'_>,
    env: &mut ShellEnvironment,
    last_status: u8,
    stderr: &mut impl Write,
) -> io::Result<Option<u8>> {
    let [raw] = simple.argv.as_slice() else {
        return Ok(None);
    };
    let Some(body) = body_of(raw) else {
        return Ok(None);
    };
    Ok(Some(run(body, env, last_status, stderr)?))
}

fn body_of(raw: &str) -> Option<&str> {
    let raw = raw.trim();
    if raw.len() < 4 || !raw.starts_with("[[") || !raw.ends_with("]]") {
        return None;
    }
    Some(raw[2..raw.len() - 2].trim())
}

fn run(
    body: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match expanded_tokens(body, env, last_status).and_then(|v| expr::eval(&v)) {
        Ok(true) => Ok(0),
        Ok(false) => Ok(1),
        Err(msg) => {
            writeln!(stderr, "[[: {msg}")?;
            Ok(2)
        }
    }
}

fn expanded_tokens(
    body: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<Vec<String>, &'static str> {
    let mut tokens = Vec::new();
    lex::tokenize_into(body, &mut tokens).map_err(|_| "parse error")?;
    tokens
        .iter()
        .map(|token| expand_token(token.kind, token.lexeme(body), env, last_status))
        .collect()
}

fn expand_token(
    kind: TokenKind,
    raw: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<String, &'static str> {
    if kind.is_operator() {
        return Ok(raw.to_owned());
    }
    expand::expand_word_for_exec(raw, env, last_status)
        .map(|word| word.into_string())
        .map_err(|_| "expansion failed")
}
