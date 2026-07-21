//! `set` builtin — assign or list shell-local variables.

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn set(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match argv.len() {
        1 => write_locals(shell_env, stdout),
        2 => {
            let spec = &argv[1];
            if let Some((name, value)) = spec.split_once('=') {
                return assign(name, value, shell_env, stderr);
            }
            assign(spec, "", shell_env, stderr)
        }
        4 if argv[2] == "=" => assign(&argv[1], &argv[3], shell_env, stderr),
        _ => {
            writeln!(stderr, "set: Too many arguments.")?;
            Ok(1)
        }
    }
}

fn assign(
    name: &str,
    value: &str,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if !is_valid_name(name) {
        writeln!(
            stderr,
            "set: Variable name must contain alphanumeric characters."
        )?;
        return Ok(1);
    }
    shell_env.set_local(name, value);
    Ok(0)
}

fn write_locals(shell_env: &ShellEnvironment, stdout: &mut impl Write) -> io::Result<u8> {
    for (name, value) in shell_env.iter_locals() {
        writeln!(stdout, "{name}={value}")?;
    }
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
