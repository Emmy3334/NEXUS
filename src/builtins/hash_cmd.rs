//! `hash` builtin — list or reset the command path cache.

use crate::builtins::BuiltinResult;
use crate::harden;
use crate::pathfind;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &crate::env::ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    let path_var = harden::effective_path(shell_env);
    match argv.get(1).map(String::as_str) {
        None => list(stdout, &path_var),
        Some("-r") if argv.len() == 2 => {
            pathfind::clear_hash();
            Ok(BuiltinResult::Status(0))
        }
        Some(flag) => {
            writeln!(stderr, "hash: invalid option — {flag}")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn list(stdout: &mut impl Write, path_var: &str) -> io::Result<BuiltinResult> {
    for (name, path) in pathfind::hash_entries(path_var) {
        writeln!(stdout, "{name}={}", path.display())?;
    }
    Ok(BuiltinResult::Status(0))
}
