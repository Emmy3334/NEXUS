//! `local` builtin — function-scoped shell locals.

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if shell_env.func_depth() == 0 {
        writeln!(stderr, "local: not in a function")?;
        return Ok(BuiltinResult::Status(1));
    }
    match argv.len() {
        1 => {
            writeln!(stderr, "local: Too few arguments.")?;
            Ok(BuiltinResult::Status(1))
        }
        2 => {
            let spec = &argv[1];
            if let Some((name, value)) = spec.split_once('=') {
                return assign(name, value, shell_env, stderr);
            }
            assign(spec, "", shell_env, stderr)
        }
        4 if argv[2] == "=" => assign(&argv[1], &argv[3], shell_env, stderr),
        _ => {
            writeln!(stderr, "local: Too many arguments.")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn assign(
    name: &str,
    value: &str,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if !is_valid_name(name) {
        writeln!(
            stderr,
            "local: Variable name must contain alphanumeric characters."
        )?;
        return Ok(BuiltinResult::Status(1));
    }
    shell_env.declare_local(name, value);
    Ok(BuiltinResult::Status(0))
}

fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
