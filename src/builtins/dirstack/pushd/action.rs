//! Stack mutations for `pushd`.

use super::super::args::parse_plus_index;
use crate::builtins::cd;
use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

pub(super) fn dispatch(
    rest: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match rest.first().map(String::as_str) {
        None => swap_top(shell_env, last_status, stdout, stderr),
        Some(raw) => match parse_plus_index(raw) {
            Some(n) => rotate_to(n, shell_env, last_status, stdout, stderr),
            None => match resolve_push_target(raw, shell_env, stderr)? {
                Ok(path) => push_path(path, shell_env, last_status, stdout, stderr),
                Err(code) => Ok(code),
            },
        },
    }
}

/// Push `target` (used by `dirs -L` as well as the builtin).
pub(crate) fn push_path(
    target: PathBuf,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let cwd = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    shell_env.dir_stack.ensure_seeded(cwd.clone());
    shell_env.dir_stack.push(cwd);
    let code = cd::change_directory(&target, shell_env, last_status, stdout, stderr)?;
    if code != 0 {
        let _ = shell_env.dir_stack.pop();
    }
    Ok(code)
}

fn swap_top(
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if shell_env.dir_stack.len() < 2 || !shell_env.dir_stack.rotate_left(1) {
        writeln!(stderr, "pushd: No other directory.")?;
        return Ok(1);
    }
    let Some(top) = shell_env.dir_stack.get(0).cloned() else {
        writeln!(stderr, "pushd: No other directory.")?;
        return Ok(1);
    };
    cd::change_directory(&top, shell_env, last_status, stdout, stderr)
}

fn rotate_to(
    n: usize,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if !shell_env.dir_stack.rotate_left(n) {
        writeln!(stderr, "pushd: Directory stack not that deep.")?;
        return Ok(1);
    }
    let Some(top) = shell_env.dir_stack.get(0).cloned() else {
        writeln!(stderr, "pushd: Directory stack not that deep.")?;
        return Ok(1);
    };
    cd::change_directory(&top, shell_env, last_status, stdout, stderr)
}

fn resolve_push_target(
    raw: &str,
    shell_env: &ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<Result<PathBuf, u8>> {
    if raw != "-" {
        return Ok(Ok(PathBuf::from(raw)));
    }
    match shell_env.get("OLDPWD") {
        Some(old) => Ok(Ok(PathBuf::from(old))),
        None => {
            writeln!(stderr, "pushd: OLDPWD not set.")?;
            Ok(Err(1))
        }
    }
}
