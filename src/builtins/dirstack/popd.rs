//! `popd` — pop or drop a stack entry and cd when needed.

use super::args::{parse_cmd_args, parse_plus_index};
use super::print::print_stack;
use crate::builtins::cd;
use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

pub(crate) fn popd_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let cwd = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    shell_env.dir_stack.ensure_seeded(cwd);
    let (flags, rest) = match parse_cmd_args("popd", argv, stderr)? {
        Ok(pair) => pair,
        Err(code) => return Ok(code),
    };
    let index = match pop_index(rest, stderr)? {
        Ok(n) => n,
        Err(code) => return Ok(code),
    };
    let code = pop_at(index, shell_env, last_status, stdout, stderr)?;
    if code != 0 {
        return Ok(code);
    }
    print_stack(shell_env, flags, stdout)
}

fn pop_index(rest: &[String], stderr: &mut impl Write) -> io::Result<Result<usize, u8>> {
    match rest.first().map(String::as_str) {
        None => Ok(Ok(0)),
        Some(raw) => match parse_plus_index(raw) {
            Some(n) => Ok(Ok(n)),
            None => {
                writeln!(stderr, "popd: Invalid argument.")?;
                Ok(Err(1))
            }
        },
    }
}

fn pop_at(
    index: usize,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if shell_env.dir_stack.len() < 2 {
        writeln!(stderr, "popd: Directory stack empty.")?;
        return Ok(1);
    }
    if shell_env.dir_stack.remove_at(index).is_none() {
        writeln!(stderr, "popd: Directory stack not that deep.")?;
        return Ok(1);
    }
    if index > 0 {
        return Ok(0);
    }
    let Some(next) = shell_env.dir_stack.get(0).cloned() else {
        writeln!(stderr, "popd: Directory stack empty.")?;
        return Ok(1);
    };
    cd::change_directory(&next, shell_env, last_status, stdout, stderr)
}
