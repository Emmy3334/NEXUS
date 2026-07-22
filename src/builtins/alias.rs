//! `alias` builtin — list or define command aliases.

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn alias(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match argv.len() {
        1 => write_all(shell_env, stdout),
        2 => write_one(shell_env, &argv[1], stdout, stderr),
        _ => define(shell_env, &argv[1], &argv[2..], stderr),
    }
}

fn write_all(shell_env: &ShellEnvironment, stdout: &mut impl Write) -> io::Result<u8> {
    for (name, body) in shell_env.iter_aliases() {
        writeln!(stdout, "{name}\t{body}")?;
    }
    Ok(0)
}

fn write_one(
    shell_env: &ShellEnvironment,
    name: &str,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match shell_env.alias_get(name) {
        Some(body) => {
            writeln!(stdout, "{name}\t{body}")?;
            Ok(0)
        }
        None => {
            writeln!(stderr, "alias: {name}: Not found.")?;
            Ok(1)
        }
    }
}

fn define(
    shell_env: &mut ShellEnvironment,
    name: &str,
    words: &[String],
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if !is_valid_name(name) {
        writeln!(
            stderr,
            "alias: Variable name must contain alphanumeric characters."
        )?;
        return Ok(1);
    }
    let body = words.join(" ");
    shell_env.alias_set(name, body);
    Ok(0)
}

fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
