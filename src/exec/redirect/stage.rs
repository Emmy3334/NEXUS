//! Running one simple command (builtin or external) against resolved redirect files.

use super::child_io::run_with_io;
use super::files::{open_redirect_files, RedirectFiles, StdinSource};
use super::heredoc::HeredocState;
use super::stage_shell;
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::exec::io::ExecIo;
use crate::exec::{exit_status_code, CommandResult, StdoutMode};
use crate::heal;
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
        return crate::exec::execute_command_mode(
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
    if argv
        .first()
        .is_some_and(|n| shell_env.function_get(n).is_some())
    {
        return stage_shell::run_function(argv, files, shell_env, last_status, io);
    }
    if argv.first().is_some_and(|n| builtins::is_builtin(n)) {
        return stage_shell::run_builtin(argv, files, shell_env, last_status, io);
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

fn execute_external_with_files(
    stdout_mode: StdoutMode,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    files: RedirectFiles,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };
    let mut command = Command::new(program);
    command.args(args).env_clear().envs(shell_env.iter());
    crate::jobs::prepare_child_command(&mut command);
    let (stdin_bytes, copy_out) = configure_stdio(&mut command, files, stdout_mode);
    run_configured(
        &mut command,
        stdin_bytes,
        copy_out,
        argv,
        shell_env,
        stdout,
        stderr,
    )
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

#[allow(clippy::too_many_arguments)]
fn run_configured(
    command: &mut Command,
    stdin_bytes: Option<Vec<u8>>,
    copy_out: bool,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if stdin_bytes.is_none() && !copy_out && crate::jobs::job_control_enabled() {
        crate::jobs::prepare_process_group(command, None);
        return spawn_job_control(command, argv, shell_env, stderr);
    }
    if stdin_bytes.is_none() && !copy_out {
        return match command.status() {
            Ok(status) => Ok(exit_status_code(status)),
            Err(err) => Ok(heal::after_spawn_failure(
                argv, &err, shell_env, stdout, stderr,
            )?),
        };
    }
    run_with_io(
        command,
        stdin_bytes,
        copy_out,
        argv,
        shell_env,
        stdout,
        stderr,
    )
}

fn spawn_job_control(
    command: &mut Command,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return heal::after_spawn_failure(argv, &err, shell_env, &mut io::sink(), stderr);
        }
    };
    match crate::jobs::wait_foreground(&mut child)? {
        crate::jobs::FgWait::Done(code) => Ok(code),
        crate::jobs::FgWait::Stopped => {
            let command = argv.join(" ");
            let (id, _) =
                shell_env
                    .jobs
                    .add(command.clone(), vec![child], crate::jobs::JobState::Stopped);
            writeln!(stderr, "[{id}]+  Suspended                 {command}")?;
            Ok(crate::jobs::sigtstp_status())
        }
    }
}
