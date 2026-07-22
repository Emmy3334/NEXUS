//! Minimal tcsh `@` arithmetic for loop counters (`@ i++`, `@ i = n`).

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn at_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match argv.len() {
        2 => bump(&argv[1], shell_env, stderr),
        4 if argv[2] == "=" => assign(&argv[1], &argv[3], shell_env, stderr),
        _ => {
            writeln!(stderr, "@: Expression Syntax.")?;
            Ok(1)
        }
    }
}

fn bump(spec: &str, env: &mut ShellEnvironment, stderr: &mut impl Write) -> io::Result<u8> {
    let (name, delta) = if let Some(name) = spec.strip_suffix("++") {
        (name, 1_i64)
    } else if let Some(name) = spec.strip_suffix("--") {
        (name, -1_i64)
    } else {
        writeln!(stderr, "@: Expression Syntax.")?;
        return Ok(1);
    };
    if !is_valid_name(name) {
        writeln!(stderr, "@: Expression Syntax.")?;
        return Ok(1);
    }
    let cur = read_int(env, name);
    env.set_local(name, (cur + delta).to_string());
    Ok(0)
}

fn assign(
    name: &str,
    value: &str,
    env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if !is_valid_name(name) {
        writeln!(stderr, "@: Expression Syntax.")?;
        return Ok(1);
    }
    let Ok(n) = value.parse::<i64>() else {
        writeln!(stderr, "@: Expression Syntax.")?;
        return Ok(1);
    };
    env.set_local(name, n.to_string());
    Ok(0)
}

fn read_int(env: &ShellEnvironment, name: &str) -> i64 {
    env.lookup(name).and_then(|v| v.parse().ok()).unwrap_or(0)
}

fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
