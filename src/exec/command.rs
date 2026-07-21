//! Builtin dispatch and external spawn (inherit or capture stdout).

use super::process::{exit_status_code, report_spawn_failure};
use super::stdout_mode::StdoutMode;
use super::CommandResult;
use crate::builtins;
use crate::env::ShellEnvironment;

use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Command, Stdio};

/// Dispatch a simple command through builtins or an external spawn.
pub fn execute_command(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    execute_command_mode(
        StdoutMode::Inherit,
        argv,
        shell_env,
        last_status,
        stdout,
        stderr,
    )
}

pub(crate) fn execute_command_mode(
    stdout_mode: StdoutMode,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if let Some(result) = builtins::try_run(argv, shell_env, last_status, stdout, stderr)? {
        return Ok(result.into());
    }
    Ok(CommandResult::Status(execute_external_mode(
        stdout_mode,
        argv,
        shell_env,
        stdout,
        stderr,
    )?))
}

/// Run an external program with the shell's environment copy.
pub fn execute_external<S: AsRef<OsStr>>(
    argv: &[S],
    shell_env: &ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    execute_external_mode(StdoutMode::Inherit, argv, shell_env, stdout, stderr)
}

pub(crate) fn execute_external_mode<S: AsRef<OsStr>>(
    stdout_mode: StdoutMode,
    argv: &[S],
    shell_env: &ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };
    let mut command = Command::new(program);
    command.args(args).env_clear().envs(shell_env.iter());
    match stdout_mode {
        StdoutMode::Inherit => match command.status() {
            Ok(status) => Ok(exit_status_code(status)),
            Err(err) => {
                let name = program.as_ref().to_string_lossy();
                report_spawn_failure(&name, &err, stderr)
            }
        },
        StdoutMode::Capture => spawn_captured(&mut command, program.as_ref(), stdout, stderr),
    }
}

fn spawn_captured(
    command: &mut Command,
    program: &OsStr,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let mut child = match command.stdout(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(err) => {
            let name = program.to_string_lossy();
            return report_spawn_failure(&name, &err, stderr);
        }
    };
    if let Some(mut pipe) = child.stdout.take() {
        io::copy(&mut pipe, stdout)?;
    }
    Ok(exit_status_code(child.wait()?))
}
