//! `|` pipeline execution — OS pipes between pipeline stages.
//!
//! Builtins / subshells in a multi-stage pipeline use isolated env (cloned;
//! `exit` does not kill the parent shell). Status is the last stage’s status.

mod builtin;
mod external;
mod prepare;
mod state;
mod subshell_stage;

use self::builtin::run_builtin_stage;
use self::external::run_external_stage;
use self::prepare::{build_stages, PreparedStage};
use self::state::PipeState;
use self::subshell_stage::run_subshell_stage;

use super::io::ExecIo;
use super::redirect::{open_redirect_files, HeredocState};
use super::{abandon_children, wait_children, CommandResult};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::jobs::{FgWait, JobState};
use crate::parse::{Pipeline, Redirect};

use std::io::{self, BufRead, Write};

/// Per-pipeline state that every stage needs but does not itself own.
pub(super) struct StageCtx<'a, I, O, E> {
    pub(super) shell_env: &'a ShellEnvironment,
    pub(super) last_status: u8,
    pub(super) heredocs: &'a mut HeredocState,
    pub(super) stdin: &'a mut I,
    pub(super) stdout: &'a mut O,
    pub(super) stderr: &'a mut E,
    pub(super) stdout_mode: super::StdoutMode,
}

/// Multi-stage `|` pipeline. Status is the last stage’s status.
pub(super) fn execute_piped_stages<I: BufRead, O: Write, E: Write>(
    pipeline: &Pipeline<'_>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    let stages = match build_stages(pipeline, shell_env, last_status, io)? {
        Ok(stages) => stages,
        Err(result) => return Ok(result),
    };
    let mut state = PipeState::new();
    if let Some(result) = run_stages(&stages, shell_env, last_status, heredocs, io, &mut state)? {
        return Ok(result);
    }
    state.drain_pending();
    finish_pipeline(pipeline, shell_env, state, io.stderr)
}

fn run_stages<I: BufRead, O: Write, E: Write>(
    stages: &[PreparedStage<'_>],
    shell_env: &ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
    state: &mut PipeState,
) -> io::Result<Option<CommandResult>> {
    let mut argv_scratch = Vec::new();
    let mut ctx = StageCtx {
        shell_env,
        last_status,
        heredocs,
        stdin: io.stdin,
        stdout: io.stdout,
        stderr: io.stderr,
        stdout_mode: io.stdout_mode,
    };
    let last_index = stages.len().saturating_sub(1);
    for (index, stage) in stages.iter().enumerate() {
        if let Some(result) = run_prepared(
            stage,
            index == last_index,
            &mut ctx,
            state,
            &mut argv_scratch,
        )? {
            return Ok(Some(result));
        }
    }
    Ok(None)
}

fn finish_pipeline(
    pipeline: &Pipeline<'_>,
    shell_env: &mut ShellEnvironment,
    mut state: PipeState,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    match state.finish()? {
        FgWait::Done(status) => Ok(CommandResult::Status(status)),
        FgWait::Stopped => {
            let command = super::render::render_pipeline(pipeline);
            let children = std::mem::take(&mut state.children);
            let (id, _) = shell_env
                .jobs
                .add(command.clone(), children, JobState::Stopped);
            writeln!(stderr, "[{id}]+  Suspended                 {command}")?;
            Ok(CommandResult::Status(crate::jobs::sigtstp_status()))
        }
    }
}

fn run_prepared<I: BufRead, O: Write, E: Write>(
    stage: &PreparedStage<'_>,
    is_last: bool,
    ctx: &mut StageCtx<'_, I, O, E>,
    state: &mut PipeState,
    argv_scratch: &mut Vec<String>,
) -> io::Result<Option<CommandResult>> {
    match stage {
        PreparedStage::Simple { argv, redirects } => {
            run_simple_stage(argv, redirects, is_last, ctx, state)
        }
        PreparedStage::Subshell { list, redirects } => {
            run_subshell_stage(list, redirects, is_last, ctx, state, argv_scratch)
        }
    }
}

fn run_simple_stage<I: BufRead, O: Write, E: Write>(
    stage: &[String],
    redirects: &[Redirect<'_>],
    is_last: bool,
    ctx: &mut StageCtx<'_, I, O, E>,
    state: &mut PipeState,
) -> io::Result<Option<CommandResult>> {
    let Some(name) = stage.first().map(String::as_str) else {
        abandon_children(&mut state.children);
        return Ok(Some(CommandResult::Status(0)));
    };
    let files = match open_redirect_files(
        redirects,
        ctx.heredocs,
        ctx.shell_env,
        ctx.last_status,
        ctx.stdin,
        ctx.stderr,
    )? {
        Ok(files) => files,
        Err(code) => {
            abandon_children(&mut state.children);
            let _ = wait_children(&mut state.children)?;
            return Ok(Some(CommandResult::Status(code)));
        }
    };
    if builtins::is_builtin(name) {
        run_builtin_stage(stage, files, is_last, ctx, state)
    } else {
        run_external_stage(name, stage, files, is_last, ctx, state)
    }
}
