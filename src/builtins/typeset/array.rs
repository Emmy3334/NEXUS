//! Array `typeset -a` assignments.

use super::super::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    args: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match args {
        [] => {
            writeln!(stderr, "typeset: Too few arguments.")?;
            Ok(BuiltinResult::Status(1))
        }
        [spec] => assign_spec(spec, shell_env, stderr),
        [name, eq, value] if eq.as_str() == "=" => assign(name, value, shell_env, stderr),
        _ => {
            writeln!(stderr, "typeset: Too many arguments.")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn assign_spec(
    spec: &str,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    if let Some((name, value)) = spec.split_once('=') {
        return assign(name, value, shell_env, stderr);
    }
    assign(spec, "", shell_env, stderr)
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
            "typeset: Variable name must contain alphanumeric characters."
        )?;
        return Ok(BuiltinResult::Status(1));
    }
    let elements = split_elements(value);
    if shell_env.func_depth() > 0 {
        shell_env.declare_array(name, elements);
    } else {
        shell_env.array_set(name, elements);
    }
    Ok(BuiltinResult::Status(0))
}

fn split_elements(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }
    value.split(':').map(str::to_owned).collect()
}

fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
