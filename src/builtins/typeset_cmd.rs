//! `typeset` — function-local like `local`, optional `-x` / `--export`.

use crate::builtins::BuiltinResult;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let (export, args) = match parse_flags(argv) {
        Ok(v) => v,
        Err(msg) => {
            writeln!(stderr, "typeset: {msg}")?;
            return Ok(BuiltinResult::Status(1));
        }
    };
    if !export && shell_env.func_depth() == 0 {
        writeln!(stderr, "typeset: not in a function")?;
        return Ok(BuiltinResult::Status(1));
    }
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

fn parse_flags(argv: &[String]) -> Result<(bool, &[String]), &'static str> {
    let rest = argv.get(1..).unwrap_or(&[]);
    match rest.first().map(String::as_str) {
        Some("-x" | "--export") => Ok((true, &rest[1..])),
        Some(flag) if flag.starts_with('-') => Err("unknown option"),
        _ => Ok((false, rest)),
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
