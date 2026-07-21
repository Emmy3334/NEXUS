//! Command execution: builtins first, then external programs.
//!
//! - [`execute_list`] walks `;`-separated pipelines
//! - [`pipe`] connects stages with OS pipes
//! - File redirections (`<` / `>` / `>>`) are lexed/parsed as errors today;
//!   their apply path will live in a dedicated `redirect` module when that
//!   Minishell2 slice lands.

mod pipe;

use crate::builtins::{self, BuiltinResult};
use crate::env::ShellEnvironment;
use crate::parse::{self, CommandList, Pipeline};

use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Child, Command, ExitStatus};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

/// Outcome of running one simple command or a list/pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "shell exit vs continue must be handled by the REPL"]
pub enum CommandResult {
    /// Keep the REPL running with this status.
    Status(u8),
    /// Terminate the shell with this status (`exit` builtin).
    Exit(u8),
}

impl From<BuiltinResult> for CommandResult {
    fn from(result: BuiltinResult) -> Self {
        match result {
            BuiltinResult::Status(code) => Self::Status(code),
            BuiltinResult::Exit(code) => Self::Exit(code),
        }
    }
}

/// Run a command list: each `;`-separated pipeline in order.
///
/// `exit` in a single-command pipeline stops the shell. Reuses `argv`
/// across simple (non-pipe) commands.
pub fn execute_list(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    for pipeline in &list.pipelines {
        match execute_pipeline(pipeline, argv, shell_env, last_status, stdout, stderr)? {
            CommandResult::Status(code) => last_status = code,
            CommandResult::Exit(code) => return Ok(CommandResult::Exit(code)),
        }
    }

    Ok(CommandResult::Status(last_status))
}

fn execute_pipeline(
    pipeline: &Pipeline<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    match pipeline.commands.as_slice() {
        [] => Ok(CommandResult::Status(last_status)),
        [simple] => {
            parse::fill_argv(&simple.argv, argv);
            if argv.is_empty() {
                return Ok(CommandResult::Status(last_status));
            }
            execute_command(argv, shell_env, last_status, stdout, stderr)
        }
        _ => pipe::execute_piped_stages(pipeline, shell_env, last_status, stdout, stderr),
    }
}

/// Dispatch a simple command through builtins or an external spawn.
pub fn execute_command(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if let Some(result) = builtins::try_run(argv, shell_env, last_status, stdout, stderr)? {
        return Ok(result.into());
    }

    Ok(CommandResult::Status(execute_external(
        argv, shell_env, stderr,
    )?))
}

/// Run an external program with the shell's environment copy.
///
/// - Resolves via absolute/relative path or `PATH`.
/// - On “not found”, writes `{name}: Command not found.` to `stderr` and
///   returns `127`.
pub fn execute_external<S: AsRef<OsStr>>(
    argv: &[S],
    shell_env: &ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };

    match Command::new(program)
        .args(args)
        .env_clear()
        .envs(shell_env.iter())
        .status()
    {
        Ok(status) => Ok(exit_status_code(status)),
        Err(err) => {
            let name = program.as_ref().to_string_lossy();
            Ok(report_spawn_failure(&name, &err, stderr)?)
        }
    }
}

pub(crate) fn report_spawn_failure(
    program: &str,
    err: &io::Error,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match err.kind() {
        io::ErrorKind::NotFound => {
            writeln!(stderr, "{program}: Command not found.")?;
            Ok(127)
        }
        io::ErrorKind::PermissionDenied => {
            writeln!(stderr, "{program}: Permission denied.")?;
            Ok(126)
        }
        _ => {
            writeln!(stderr, "{program}: {err}")?;
            Ok(1)
        }
    }
}

pub(crate) fn exit_status_code(status: ExitStatus) -> u8 {
    if let Some(code) = status.code() {
        return code as u8;
    }

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        return 128u8.saturating_add(signal as u8);
    }

    1
}

pub(crate) fn build_external_command(argv: &[String], shell_env: &ShellEnvironment) -> Command {
    let mut command = Command::new(&argv[0]);
    if argv.len() > 1 {
        command.args(&argv[1..]);
    }
    command.env_clear().envs(shell_env.iter());
    command
}

pub(crate) fn abandon_children(children: &mut [Child]) {
    for child in children.iter_mut() {
        let _ = child.kill();
    }
}

pub(crate) fn wait_children(children: &mut Vec<Child>) -> io::Result<u8> {
    let count = children.len();
    let mut last_status = 0u8;
    for (index, mut child) in children.drain(..).enumerate() {
        let status = child.wait()?;
        if index + 1 == count {
            last_status = exit_status_code(status);
        }
    }
    Ok(last_status)
}

/// Run a builtin and map `exit` to a status (pipeline / subshell semantics).
pub(crate) fn run_builtin_status(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match builtins::try_run(argv, shell_env, last_status, stdout, stderr)? {
        Some(BuiltinResult::Status(code) | BuiltinResult::Exit(code)) => Ok(code),
        None => Ok(0),
    }
}
