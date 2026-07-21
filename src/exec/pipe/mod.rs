//! `|` pipeline execution — OS pipes between simple-command stages.
//!
//! Builtins in a multi-stage pipeline use subshell semantics (cloned env;
//! `exit` does not kill the parent shell). Status is the last stage’s status.
//! File / heredoc redirects on a stage override the pipe on that fd.

mod builtin;
mod external;
mod state;

use self::builtin::run_builtin_stage;
use self::external::run_external_stage;
use self::state::PipeState;

use super::io::ExecIo;
use super::redirect::{open_redirect_files, HeredocState};
use super::stdout_mode::StdoutMode;
use super::{abandon_children, wait_children, CommandResult};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::{Pipeline, Redirect};

use std::io::{self, BufRead, Write};

type Stage<'a> = (Vec<String>, &'a [Redirect<'a>]);

/// Per-pipeline state that every stage needs but does not itself own.
pub(super) struct StageCtx<'a, I, O, E> {
    pub(super) shell_env: &'a ShellEnvironment,
    pub(super) last_status: u8,
    pub(super) heredocs: &'a mut HeredocState,
    pub(super) stdin: &'a mut I,
    pub(super) stdout: &'a mut O,
    pub(super) stderr: &'a mut E,
    pub(super) stdout_mode: StdoutMode,
}

fn build_stages<'a, I: BufRead, O: Write, E: Write>(
    pipeline: &'a Pipeline<'a>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<Result<Vec<Stage<'a>>, CommandResult>> {
    let mut stages = Vec::with_capacity(pipeline.commands.len());
    let mut fields = Vec::new();
    for command in &pipeline.commands {
        let mut argv = Vec::with_capacity(command.argv.len());
        for word in &command.argv {
            if let Err(result) =
                expand_stage_word(word, shell_env, last_status, io, &mut fields, &mut argv)?
            {
                return Ok(Err(result));
            }
        }
        if let Err(err) =
            crate::alias::apply_aliases(&mut argv, shell_env, last_status, io.stdin, io.stderr)
        {
            writeln!(io.stderr, "{}", err.message())?;
            return Ok(Err(CommandResult::Status(1)));
        }
        stages.push((argv, command.redirects.as_slice()));
    }
    Ok(Ok(stages))
}

fn expand_stage_word<I: BufRead, O: Write, E: Write>(
    word: &str,
    shell_env: &ShellEnvironment,
    last_status: u8,
    io: &mut ExecIo<'_, I, O, E>,
    fields: &mut Vec<crate::expand::ExpandedWord>,
    argv: &mut Vec<String>,
) -> io::Result<Result<(), CommandResult>> {
    let mut capture = |body: &str| {
        crate::exec::capture_command_output(body, shell_env, last_status, io.stdin, io.stderr)
    };
    match crate::expand::expand_word_fields_into(word, shell_env, last_status, fields, &mut capture)
    {
        Ok(()) => {
            for field in fields.drain(..) {
                argv.extend(crate::glob::expand_globs(&field));
            }
            Ok(Ok(()))
        }
        Err(err) => {
            writeln!(io.stderr, "{}", err.message())?;
            Ok(Err(CommandResult::Status(1)))
        }
    }
}

fn run_stage<I: BufRead, O: Write, E: Write>(
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

/// Multi-stage `|` pipeline. Status is the last stage’s status.
pub(super) fn execute_piped_stages<I: BufRead, O: Write, E: Write>(
    pipeline: &Pipeline<'_>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    let stages = match build_stages(pipeline, shell_env, last_status, io)? {
        Ok(stages) => stages,
        Err(result) => return Ok(result),
    };
    let stdout_mode = io.stdout_mode;
    let mut state = PipeState::new();
    let mut ctx = StageCtx {
        shell_env,
        last_status,
        heredocs,
        stdin: io.stdin,
        stdout: io.stdout,
        stderr: io.stderr,
        stdout_mode,
    };
    let last_index = stages.len().saturating_sub(1);
    for (index, (stage, redirects)) in stages.iter().enumerate() {
        let is_last = index == last_index;
        if let Some(result) = run_stage(stage, redirects, is_last, &mut ctx, &mut state)? {
            return Ok(result);
        }
    }
    state.drain_pending();
    Ok(CommandResult::Status(state.finish()?))
}
