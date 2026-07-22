//! Launch a pipeline in the background and register it in the job table.

mod shell;
mod spawn;

use self::spawn::spawn_simple;
use super::io::ExecIo;
use super::list;
use super::redirect::HeredocState;
use super::CommandResult;
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::{Pipeline, PipelineCommand};

use std::io::{self, BufRead, Write};

/// Run `pipeline` asynchronously in its own process group.
pub(super) fn execute_background<I: BufRead, O: Write, E: Write>(
    pipeline: &Pipeline<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    match pipeline.commands.as_slice() {
        [PipelineCommand::Simple(simple)] => {
            background_simple(simple, argv, shell_env, last_status, heredocs, io)
        }
        _ => shell::spawn_pipeline(pipeline, shell_env, heredocs, io.stderr),
    }
}

fn background_simple<I: BufRead, O: Write, E: Write>(
    simple: &crate::parse::SimpleCommand<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    if let Err(code) = list::expand_for_background(&simple.argv, argv, shell_env, last_status, io)?
    {
        return Ok(CommandResult::Status(code));
    }
    if argv.is_empty() {
        return Ok(CommandResult::Status(0));
    }
    if builtins::is_builtin(&argv[0]) {
        let pipeline = crate::parse::Pipeline {
            commands: vec![crate::parse::PipelineCommand::Simple(simple.clone())],
            background: false,
            join: crate::parse::PipelineJoin::Seq,
        };
        return shell::spawn_pipeline(&pipeline, shell_env, heredocs, io.stderr);
    }
    spawn_simple(
        argv,
        &simple.redirects,
        shell_env,
        last_status,
        heredocs,
        io,
    )
}
