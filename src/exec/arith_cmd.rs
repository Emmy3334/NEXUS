//! Bash/ksh-style `((expr))` arithmetic command (side effects + status).

use crate::env::ShellEnvironment;
use crate::expand;
use crate::parse::SimpleCommand;

use std::io::{self, Write};

/// If `simple` is a lone `((…))` word, evaluate and return status; else `None`.
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
    if raw.len() < 4 || !raw.starts_with("((") || !raw.ends_with("))") {
        return None;
    }
    Some(&raw[2..raw.len() - 2])
}

fn run(
    body: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match expand::evaluate(body, env, last_status) {
        Ok(0) => Ok(1),
        Ok(_) => Ok(0),
        Err(err) => {
            writeln!(stderr, "{}", err.message())?;
            Ok(1)
        }
    }
}
