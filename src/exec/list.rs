//! `;`-separated command lists and single-command pipelines.

use super::io::ExecIo;
use super::pipe;
use super::redirect::{self, HeredocState};
use super::stdout_mode::StdoutMode;
use super::CommandResult;
use crate::env::ShellEnvironment;
use crate::parse::{self, CommandList, Pipeline, SimpleCommand};

use std::io::{self, BufRead, Write};

/// Run a command list with inherited external stdout (interactive / normal).
#[allow(clippy::too_many_arguments)] // thin wrapper over ExecIo
pub fn execute_list(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredoc_bodies: Vec<String>,
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let mut io = ExecIo {
        stdin,
        stdout,
        stderr,
        stdout_mode: StdoutMode::Inherit,
    };
    execute_list_with(list, argv, shell_env, last_status, heredoc_bodies, &mut io)
}

/// Run a command list piping external stdout into `stdout` (`` `…` ``).
#[allow(clippy::too_many_arguments)] // thin wrapper over ExecIo
pub(crate) fn execute_list_captured(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredoc_bodies: Vec<String>,
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let mut io = ExecIo {
        stdin,
        stdout,
        stderr,
        stdout_mode: StdoutMode::Capture,
    };
    execute_list_with(list, argv, shell_env, last_status, heredoc_bodies, &mut io)
}

fn execute_list_with<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    heredoc_bodies: Vec<String>,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    let mut heredocs = HeredocState::new(heredoc_bodies);
    for pipeline in &list.pipelines {
        match execute_pipeline(pipeline, argv, shell_env, last_status, &mut heredocs, io)? {
            CommandResult::Status(code) => last_status = code,
            CommandResult::Exit(code) => return Ok(CommandResult::Exit(code)),
        }
    }
    Ok(CommandResult::Status(last_status))
}

fn execute_pipeline<I: BufRead, O: Write, E: Write>(
    pipeline: &Pipeline<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    match pipeline.commands.as_slice() {
        [] => Ok(CommandResult::Status(last_status)),
        [simple] => run_simple(simple, argv, shell_env, last_status, heredocs, io),
        _ => pipe::execute_piped_stages(pipeline, shell_env, last_status, heredocs, io),
    }
}

fn run_simple<I: BufRead, O: Write, E: Write>(
    simple: &SimpleCommand<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    if let Err(code) = expand_simple_argv(&simple.argv, argv, shell_env, last_status, io)? {
        return Ok(CommandResult::Status(code));
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
        io,
    )
}

fn expand_simple_argv<I: BufRead, O: Write, E: Write>(
    words: &[&str],
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<Result<(), u8>> {
    if let Err(err) = parse::fill_argv(words, argv, shell_env, last_status, io.stdin, io.stderr) {
        writeln!(io.stderr, "{}", err.message())?;
        return Ok(Err(1));
    }
    if let Err(err) = crate::alias::apply_aliases(argv, shell_env, last_status, io.stdin, io.stderr)
    {
        writeln!(io.stderr, "{}", err.message())?;
        return Ok(Err(1));
    }
    Ok(Ok(()))
}
