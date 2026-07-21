//! Running one simple command (builtin or external) against resolved redirect files.

use super::files::{open_redirect_files, RedirectFiles, StdinSource};
use super::heredoc::HeredocState;
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::exec::io::ExecIo;
use crate::exec::{
    execute_command_mode, exit_status_code, report_spawn_failure, CommandResult, StdoutMode,
};
use crate::parse::Redirect;

use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

/// Run one simple command, applying file / heredoc redirects when present.
pub(in crate::exec) fn execute_simple<I: BufRead, O: Write, E: Write>(
    argv: &[String],
    redirects: &[Redirect<'_>],
    heredocs: &mut HeredocState,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    if redirects.is_empty() {
        return execute_command_mode(
            io.stdout_mode,
            argv,
            shell_env,
            last_status,
            io.stdout,
            io.stderr,
        );
    }
    let files = match open_redirect_files(
        redirects,
        heredocs,
        shell_env,
        last_status,
        io.stdin,
        io.stderr,
    )? {
        Ok(files) => files,
        Err(code) => return Ok(CommandResult::Status(code)),
    };
    dispatch_with_files(argv, files, shell_env, last_status, io)
}

fn dispatch_with_files<I: BufRead, O: Write, E: Write>(
    argv: &[String],
    files: RedirectFiles,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    if argv.first().is_some_and(|n| builtins::is_builtin(n)) {
        return execute_builtin_with_files(argv, files, shell_env, last_status, io);
    }
    Ok(CommandResult::Status(execute_external_with_files(
        io.stdout_mode,
        argv,
        shell_env,
        files,
        io.stdout,
        io.stderr,
    )?))
}

fn execute_builtin_with_files<I: BufRead, O: Write, E: Write>(
    argv: &[String],
    files: RedirectFiles,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    drop(files.stdin);
    let result = match files.stdout {
        Some(mut file) => builtins::try_run(argv, shell_env, last_status, &mut file, io.stderr)?,
        None => builtins::try_run(argv, shell_env, last_status, io.stdout, io.stderr)?,
    };
    Ok(result
        .map(CommandResult::from)
        .unwrap_or(CommandResult::Status(0)))
}

fn execute_external_with_files(
    stdout_mode: StdoutMode,
    argv: &[String],
    shell_env: &ShellEnvironment,
    files: RedirectFiles,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };
    let mut command = Command::new(program);
    command.args(args).env_clear().envs(shell_env.iter());
    let (stdin_bytes, copy_out) = configure_stdio(&mut command, files, stdout_mode);
    run_configured(&mut command, stdin_bytes, copy_out, program, stdout, stderr)
}

fn configure_stdio(
    command: &mut Command,
    files: RedirectFiles,
    stdout_mode: StdoutMode,
) -> (Option<Vec<u8>>, bool) {
    let stdin_bytes = match files.stdin {
        Some(StdinSource::File(file)) => {
            command.stdin(Stdio::from(file));
            None
        }
        Some(StdinSource::Bytes(bytes)) => {
            command.stdin(Stdio::piped());
            Some(bytes)
        }
        None => None,
    };
    let copy_out = match files.stdout {
        Some(file) => {
            command.stdout(Stdio::from(file));
            false
        }
        None if stdout_mode == StdoutMode::Capture => {
            command.stdout(Stdio::piped());
            true
        }
        None => false,
    };
    (stdin_bytes, copy_out)
}

fn run_configured(
    command: &mut Command,
    stdin_bytes: Option<Vec<u8>>,
    copy_out: bool,
    program: &str,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if stdin_bytes.is_none() && !copy_out {
        return match command.status() {
            Ok(status) => Ok(exit_status_code(status)),
            Err(err) => Ok(report_spawn_failure(program, &err, stderr)?),
        };
    }
    match command.spawn() {
        Ok(mut child) => {
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
        Err(err) => Ok(report_spawn_failure(program, &err, stderr)?),
    }
}
