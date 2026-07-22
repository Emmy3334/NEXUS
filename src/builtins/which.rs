//! `which` / `where` — locate commands on PATH or as builtins.

use crate::builtins;
use crate::env::ShellEnvironment;
use crate::pathfind;

use std::io::{self, Write};

pub(super) fn which_cmd(
    argv: &[String],
    shell_env: &ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    locate(argv, shell_env, false, stdout, stderr)
}

pub(super) fn where_cmd(
    argv: &[String],
    shell_env: &ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    locate(argv, shell_env, true, stdout, stderr)
}

fn locate(
    argv: &[String],
    shell_env: &ShellEnvironment,
    all: bool,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let names = &argv[1..];
    if names.is_empty() {
        writeln!(stderr, "{}: Too few arguments.", argv[0])?;
        return Ok(1);
    }
    let path_var = shell_env.get("PATH").unwrap_or("");
    let mut status = 0_u8;
    for name in names {
        if !print_one(name, path_var, all, stdout)? {
            writeln!(stderr, "{name}: Command not found.")?;
            status = 1;
        }
    }
    Ok(status)
}

fn print_one(name: &str, path_var: &str, all: bool, stdout: &mut impl Write) -> io::Result<bool> {
    let mut found = false;
    if builtins::is_builtin(name) {
        writeln!(stdout, "{name}: shell built-in command.")?;
        found = true;
        if !all {
            return Ok(true);
        }
    }
    if all {
        for path in pathfind::resolve_all(name, path_var) {
            writeln!(stdout, "{}", path.display())?;
            found = true;
        }
    } else if let Some(path) = pathfind::resolve_first(name, path_var) {
        writeln!(stdout, "{}", path.display())?;
        found = true;
    }
    Ok(found)
}
