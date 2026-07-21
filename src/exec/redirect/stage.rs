//! Running one simple command (builtin or external) against resolved redirect files.

use super::files::{open_redirect_files, RedirectFiles, StdinSource};
use super::heredoc::HeredocState;
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::exec::{exit_status_code, report_spawn_failure, CommandResult};
use crate::parse::Redirect;

use std::io::{self, Write};
use std::process::{Command, Stdio};

/// Run one simple command, applying file / heredoc redirects when present.
pub(in crate::exec) fn execute_simple(
    argv: &[String],
    redirects: &[Redirect<'_>],
    heredocs: &mut HeredocState,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if redirects.is_empty() {
        return crate::exec::execute_command(argv, shell_env, last_status, stdout, stderr);
    }

    let files = match open_redirect_files(redirects, heredocs, shell_env, last_status, stderr)? {
        Ok(files) => files,
        Err(code) => return Ok(CommandResult::Status(code)),
    };

    if let Some(name) = argv.first().map(String::as_str) {
        if builtins::is_builtin(name) {
            return execute_builtin_with_files(argv, files, shell_env, last_status, stdout, stderr);
        }
    }

    Ok(CommandResult::Status(execute_external_with_files(
        argv, shell_env, files, stderr,
    )?))
}

fn execute_builtin_with_files(
    argv: &[String],
    files: RedirectFiles,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    // Builtins don't consume stdin yet; drop any stdin redirect (path already validated).
    drop(files.stdin);

    let result = match files.stdout {
        Some(mut file) => builtins::try_run(argv, shell_env, last_status, &mut file, stderr)?,
        None => builtins::try_run(argv, shell_env, last_status, stdout, stderr)?,
    };

    Ok(result
        .map(CommandResult::from)
        .unwrap_or(CommandResult::Status(0)))
}

fn execute_external_with_files(
    argv: &[String],
    shell_env: &ShellEnvironment,
    files: RedirectFiles,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };

    let mut command = Command::new(program);
    command.args(args).env_clear().envs(shell_env.iter());

    let stdin_bytes = configure_stdio(&mut command, files);
    run_configured(&mut command, stdin_bytes, program, stderr)
}

fn configure_stdio(command: &mut Command, files: RedirectFiles) -> Option<Vec<u8>> {
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
    if let Some(file) = files.stdout {
        command.stdout(Stdio::from(file));
    }
    stdin_bytes
}

fn run_configured(
    command: &mut Command,
    stdin_bytes: Option<Vec<u8>>,
    program: &str,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match stdin_bytes {
        None => match command.status() {
            Ok(status) => Ok(exit_status_code(status)),
            Err(err) => Ok(report_spawn_failure(program, &err, stderr)?),
        },
        Some(bytes) => match command.spawn() {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(&bytes)?;
                }
                let status = child.wait()?;
                Ok(exit_status_code(status))
            }
            Err(err) => Ok(report_spawn_failure(program, &err, stderr)?),
        },
    }
}
