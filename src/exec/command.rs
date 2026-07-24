//! Builtin dispatch and external spawn (inherit or capture stdout).

use super::process::exit_status_code;
use super::stdout_mode::StdoutMode;
use super::CommandResult;
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::heal;
use crate::jobs::{self, FgWait, JobState};

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
    if let Some(result) = crate::functions::try_run(argv, shell_env, last_status, stdout, stderr)? {
        return Ok(result);
    }
    if let Some(result) = builtins::try_run(argv, shell_env, last_status, stdout, stderr)? {
        return match result {
            builtins::BuiltinResult::Repeat { count, argv } => super::repeat::run_repeat(
                stdout_mode,
                count,
                &argv,
                shell_env,
                last_status,
                stdout,
                stderr,
            ),
            other => Ok(other.into()),
        };
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
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    execute_external_mode(StdoutMode::Inherit, argv, shell_env, stdout, stderr)
}

pub(crate) fn execute_external_mode<S: AsRef<OsStr>>(
    stdout_mode: StdoutMode,
    argv: &[S],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.is_empty() {
        return Ok(0);
    }
    let owned: Vec<String> = argv
        .iter()
        .map(|a| a.as_ref().to_string_lossy().into_owned())
        .collect();
    let mut command = match super::process::build_external_command(&owned, shell_env) {
        Ok(command) => command,
        Err(msg) => {
            writeln!(stderr, "{msg}")?;
            return Ok(super::process::TRUSTED_DENY_STATUS);
        }
    };
    match stdout_mode {
        StdoutMode::Inherit => {
            jobs::prepare_process_group(&mut command, None, shell_env);
            spawn_inherit(&mut command, argv, shell_env, stdout, stderr)
        }
        StdoutMode::Capture => spawn_captured(&mut command, argv, shell_env, stdout, stderr),
    }
}

fn spawn_inherit<S: AsRef<OsStr>>(
    command: &mut Command,
    argv: &[S],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if !jobs::job_control_enabled() {
        return match command.status() {
            Ok(status) => Ok(exit_status_code(status)),
            Err(err) => heal::after_spawn_failure_os(argv, &err, shell_env, stdout, stderr),
        };
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return heal::after_spawn_failure_os(argv, &err, shell_env, stdout, stderr);
        }
    };
    match jobs::wait_foreground(&mut child)? {
        FgWait::Done(code) => Ok(code),
        FgWait::Stopped => register_stopped_external(argv, child, shell_env, stderr),
    }
}

fn register_stopped_external<S: AsRef<OsStr>>(
    argv: &[S],
    child: std::process::Child,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let command = argv
        .iter()
        .map(|arg| arg.as_ref().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    let (id, _) = shell_env
        .jobs
        .add(command.clone(), vec![child], JobState::Stopped);
    writeln!(stderr, "[{id}]+  Suspended                 {command}")?;
    Ok(jobs::sigtstp_status())
}

fn spawn_captured<S: AsRef<OsStr>>(
    command: &mut Command,
    argv: &[S],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let mut child = match command.stdout(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(err) => {
            return heal::after_spawn_failure_os(argv, &err, shell_env, stdout, stderr);
        }
    };
    if let Some(mut pipe) = child.stdout.take() {
        io::copy(&mut pipe, stdout)?;
    }
    Ok(exit_status_code(child.wait()?))
}
