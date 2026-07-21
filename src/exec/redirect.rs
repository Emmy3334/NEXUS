//! File redirections (`<`, `>`, `>>`) for simple commands and pipeline stages.
//!
//! File redirects override a pipe on the same fd. Heredoc (`<<`) is not applied
//! here — parse rejects it until that slice.

use super::{exit_status_code, report_spawn_failure, CommandResult};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::{Redirect, RedirectKind};

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub(super) struct RedirectFiles {
    pub(super) stdin: Option<File>,
    pub(super) stdout: Option<File>,
}

/// Open redirect targets. On open failure, writes to `stderr` and returns `Err(1)`.
pub(super) fn open_redirect_files(
    redirects: &[Redirect<'_>],
    stderr: &mut impl Write,
) -> io::Result<Result<RedirectFiles, u8>> {
    let mut stdin = None;
    let mut stdout = None;

    for redirect in redirects {
        match redirect.kind {
            RedirectKind::Read => match File::open(redirect.path) {
                Ok(file) => stdin = Some(file),
                Err(err) => {
                    writeln!(stderr, "{}: {err}", redirect.path)?;
                    return Ok(Err(1));
                }
            },
            RedirectKind::Write => match File::create(redirect.path) {
                Ok(file) => stdout = Some(file),
                Err(err) => {
                    writeln!(stderr, "{}: {err}", redirect.path)?;
                    return Ok(Err(1));
                }
            },
            RedirectKind::Append => {
                match OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(redirect.path)
                {
                    Ok(file) => stdout = Some(file),
                    Err(err) => {
                        writeln!(stderr, "{}: {err}", redirect.path)?;
                        return Ok(Err(1));
                    }
                }
            }
        }
    }

    Ok(Ok(RedirectFiles { stdin, stdout }))
}

pub(super) fn apply_stdout_for_stage(
    command: &mut Command,
    stdout_file: Option<File>,
    is_last: bool,
) {
    if let Some(file) = stdout_file {
        command.stdout(Stdio::from(file));
    } else if !is_last {
        command.stdout(Stdio::piped());
    }
}

/// Run one simple command, applying file redirects when present.
pub(super) fn execute_simple(
    argv: &[String],
    redirects: &[Redirect<'_>],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if redirects.is_empty() {
        return super::execute_command(argv, shell_env, last_status, stdout, stderr);
    }

    let files = match open_redirect_files(redirects, stderr)? {
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
    // Builtins don't consume stdin yet; drop the File (open already validated the path).
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
    if let Some(file) = files.stdin {
        command.stdin(Stdio::from(file));
    }
    if let Some(file) = files.stdout {
        command.stdout(Stdio::from(file));
    }

    match command.status() {
        Ok(status) => Ok(exit_status_code(status)),
        Err(err) => Ok(report_spawn_failure(program, &err, stderr)?),
    }
}
