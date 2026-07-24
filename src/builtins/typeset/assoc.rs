//! Associative-array `typeset -A` assignments.
//!
//! Literal form: `typeset -A colors=red:ff0000,green:00ff00` (pairs by `,`,
//! `key:value` by the first `:`). Later duplicate keys win.

use super::super::BuiltinResult;
use crate::env::ShellEnvironment;

use std::collections::BTreeMap;
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
    let map = parse_pairs(value);
    if shell_env.func_depth() > 0 {
        shell_env.declare_assoc(name, map);
    } else {
        shell_env.assoc_set(name, map);
    }
    Ok(BuiltinResult::Status(0))
}

fn parse_pairs(value: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if value.is_empty() {
        return map;
    }
    for pair in value.split(',') {
        let (key, val) = pair.split_once(':').unwrap_or((pair, ""));
        map.insert(key.to_owned(), val.to_owned());
    }
    map
}

fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
