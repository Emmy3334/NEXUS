//! Feed/capture helpers for a redirected external child.

use crate::env::ShellEnvironment;
use crate::exec::exit_status_code;
use crate::heal;

use std::io::{self, Write};
use std::process::Command;

pub(super) fn run_with_io(
    command: &mut Command,
    stdin_bytes: Option<Vec<u8>>,
    copy_out: bool,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return heal::after_spawn_failure(argv, &err, shell_env, stdout, stderr);
        }
    };
    if let Some(bytes) = stdin_bytes {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&bytes)?;
        }
    }
    if copy_out {
        if let Some(mut pipe) = child.stdout.take() {
            io::copy(&mut pipe, stdout)?;
        }
    }
    Ok(exit_status_code(child.wait()?))
}
