//! `env` builtin — print the shell environment (sorted).

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn env_cmd(
    argv: &[String],
    shell_env: &ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() != 1 {
        writeln!(stderr, "env: Too many arguments.")?;
        return Ok(1);
    }
    write_env(shell_env, stdout)
}

/// Print all assignments (also used by bare `setenv`).
pub(super) fn write_env(shell_env: &ShellEnvironment, stdout: &mut impl Write) -> io::Result<u8> {
    for (name, value) in shell_env.iter() {
        writeln!(stdout, "{name}={value}")?;
    }
    Ok(0)
}
