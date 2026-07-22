//! `pushd` — push directory and cd.

use crate::builtins::cd;
use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

pub(crate) fn pushd_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() > 2 {
        writeln!(stderr, "pushd: Too many arguments.")?;
        return Ok(1);
    }
    let cwd = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    shell_env.dir_stack.ensure_seeded(cwd);
    if argv.len() == 1 {
        return swap_top(shell_env, last_status, stdout, stderr);
    }
    let target = PathBuf::from(&argv[1]);
    let code = cd::change_directory(&target, shell_env, last_status, stdout, stderr)?;
    if code != 0 {
        return Ok(code);
    }
    let new_cwd = process_env::current_dir().unwrap_or(target);
    shell_env.dir_stack.push(new_cwd);
    print_stack(shell_env, stdout)
}

fn swap_top(
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if shell_env.dir_stack.len() < 2 {
        writeln!(stderr, "pushd: No other directory.")?;
        return Ok(1);
    }
    let first = shell_env.dir_stack.pop().expect("len >= 2");
    let second = shell_env.dir_stack.pop().expect("len >= 2");
    shell_env.dir_stack.push(first);
    shell_env.dir_stack.push(second.clone());
    let code = cd::change_directory(&second, shell_env, last_status, stdout, stderr)?;
    if code != 0 {
        return Ok(code);
    }
    print_stack(shell_env, stdout)
}

fn print_stack(shell_env: &ShellEnvironment, stdout: &mut impl Write) -> io::Result<u8> {
    for (i, path) in shell_env.dir_stack.iter().enumerate() {
        if i > 0 {
            write!(stdout, " ")?;
        }
        write!(stdout, "{}", path.display())?;
    }
    writeln!(stdout)?;
    Ok(0)
}
