//! `setenv` builtin — set or print environment variables.

use super::env;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn setenv(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match argv.len() {
        1 => env::write_env(shell_env, stdout),
        2 | 3 => {
            let name = &argv[1];
            if !is_valid_env_name(name) {
                writeln!(
                    stderr,
                    "setenv: Variable name must contain alphanumeric characters."
                )?;
                return Ok(1);
            }
            let value = argv.get(2).map(String::as_str).unwrap_or("");
            shell_env.set(name.clone(), value.to_owned());
            Ok(0)
        }
        _ => {
            writeln!(stderr, "setenv: Too many arguments.")?;
            Ok(1)
        }
    }
}

fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
