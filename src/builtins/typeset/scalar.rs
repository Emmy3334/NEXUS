//! Scalar `typeset` assignments.

use super::super::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    export: bool,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match args {
        [] => {
            writeln!(stderr, "typeset: Too few arguments.")?;
            Ok(BuiltinResult::Status(1))
        }
        [spec] => assign_spec(spec, export, shell_env, stderr),
        [name, eq, value] if eq.as_str() == "=" => assign(name, value, export, shell_env, stderr),
        _ => {
            writeln!(stderr, "typeset: Too many arguments.")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn assign_spec(
    spec: &str,
    export: bool,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if let Some((name, value)) = spec.split_once('=') {
        return assign(name, value, export, shell_env, stderr);
    }
    assign(spec, "", export, shell_env, stderr)
}

fn assign(
    name: &str,
    value: &str,
    export: bool,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if !is_valid_name(name) {
        writeln!(
            stderr,
            "typeset: Variable name must contain alphanumeric characters."
        )?;
        return Ok(BuiltinResult::Status(1));
    }
    if export {
        if shell_env.func_depth() > 0 {
            shell_env.declare_local_export(name, value);
        } else {
            shell_env.set(name, value);
        }
    } else {
        shell_env.declare_local(name, value);
    }
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
