//! Command execution: builtins first, then external programs.
//!
//! - [`execute_list`] walks `;`-separated pipelines
//! - [`pipe`] connects stages with OS pipes
//! - [`redirect`] applies `<` / `>` / `>>` / `<<` (overrides a pipe on that fd)
//! - [`process`] holds the child-process helpers shared by the above

mod pipe;
mod process;
mod redirect;

pub use redirect::collect_heredoc_bodies;

pub(crate) use process::{
    abandon_children, build_external_command, exit_status_code, report_spawn_failure,
    run_builtin_status, wait_children,
};

use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::{self, CommandList, Pipeline};

use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::Command;

/// Outcome of running one simple command or a list/pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "shell exit vs continue must be handled by the REPL"]
pub enum CommandResult {
    /// Keep the REPL running with this status.
    Status(u8),
    /// Terminate the shell with this status (`exit` builtin).
    Exit(u8),
}

impl From<builtins::BuiltinResult> for CommandResult {
    fn from(result: builtins::BuiltinResult) -> Self {
        match result {
            builtins::BuiltinResult::Status(code) => Self::Status(code),
            builtins::BuiltinResult::Exit(code) => Self::Exit(code),
        }
    }
}

/// Run a command list: each `;`-separated pipeline in order.
///
/// `heredoc_bodies` must contain one entry per `<<` in `list`, in left-to-right
/// appearance order (see [`collect_heredoc_bodies`]). Bodies are consumed
/// (moved) during apply so the payload is not cloned.
///
/// `exit` in a single-command pipeline stops the shell. Reuses `argv`
/// across simple (non-pipe) commands.
pub fn execute_list(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    heredoc_bodies: Vec<String>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let mut heredocs = redirect::HeredocState::new(heredoc_bodies);
    for pipeline in &list.pipelines {
        match execute_pipeline(
            pipeline,
            argv,
            shell_env,
            last_status,
            &mut heredocs,
            stdout,
            stderr,
        )? {
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
    heredocs: &mut redirect::HeredocState,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    match pipeline.commands.as_slice() {
        [] => Ok(CommandResult::Status(last_status)),
        [simple] => run_simple_pipeline(
            simple,
            argv,
            shell_env,
            last_status,
            heredocs,
            stdout,
            stderr,
        ),
        _ => pipe::execute_piped_stages(pipeline, shell_env, last_status, heredocs, stdout, stderr),
    }
}

fn run_simple_pipeline(
    simple: &parse::SimpleCommand<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut redirect::HeredocState,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if let Err(err) = parse::fill_argv(&simple.argv, argv, shell_env, last_status) {
        writeln!(stderr, "{}", err.message())?;
        return Ok(CommandResult::Status(1));
    }
    if let Err(err) = crate::alias::apply_aliases(argv, shell_env, last_status) {
        writeln!(stderr, "{}", err.message())?;
        return Ok(CommandResult::Status(1));
    }
    if argv.is_empty() {
        return Ok(CommandResult::Status(last_status));
    }
    redirect::execute_simple(
        argv,
        &simple.redirects,
        heredocs,
        shell_env,
        last_status,
        stdout,
        stderr,
    )
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
