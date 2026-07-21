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

use super::redirect::{open_redirect_files, HeredocState};
use super::{abandon_children, wait_children, CommandResult};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::{Pipeline, Redirect};

use std::io::{self, Write};

type Stage<'a> = (Vec<String>, &'a [Redirect<'a>]);

/// Per-pipeline state that every stage needs but does not itself own:
/// the (unchanging) shell env / exit status, heredoc bodies, and the REPL's
/// own stdout / stderr writers.
struct StageCtx<'a, O, E> {
    shell_env: &'a ShellEnvironment,
    last_status: u8,
    heredocs: &'a mut HeredocState,
    stdout: &'a mut O,
    stderr: &'a mut E,
}

/// Expand every command's argv (quotes/`$`/globs) into pipeline stages.
fn build_stages<'a>(
    pipeline: &'a Pipeline<'a>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    stderr: &mut impl Write,
) -> io::Result<Result<Vec<Stage<'a>>, CommandResult>> {
    let mut stages = Vec::with_capacity(pipeline.commands.len());
    for command in &pipeline.commands {
        let mut argv = Vec::with_capacity(command.argv.len());
        for word in &command.argv {
            match crate::expand::expand_word_for_exec(word, shell_env, last_status) {
                Ok(expanded) => argv.extend(crate::glob::expand_globs(&expanded)),
                Err(err) => {
                    writeln!(stderr, "{}", err.message())?;
                    return Ok(Err(CommandResult::Status(1)));
                }
            }
        }
        if let Err(err) = crate::alias::apply_aliases(&mut argv, shell_env, last_status) {
            writeln!(stderr, "{}", err.message())?;
            return Ok(Err(CommandResult::Status(1)));
        }
        stages.push((argv, command.redirects.as_slice()));
    }
    Ok(Ok(stages))
}

/// Open this stage's redirects, then dispatch to the builtin or external runner.
fn run_stage<O: Write, E: Write>(
    stage: &[String],
    redirects: &[Redirect<'_>],
    is_last: bool,
    ctx: &mut StageCtx<'_, O, E>,
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
pub(super) fn execute_piped_stages(
    pipeline: &Pipeline<'_>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let stages = match build_stages(pipeline, shell_env, last_status, stderr)? {
        Ok(stages) => stages,
        Err(result) => return Ok(result),
    };

    let mut state = PipeState::new();
    let mut ctx = StageCtx {
        shell_env,
        last_status,
        heredocs,
        stdout,
        stderr,
    };
    let last_index = stages.len().saturating_sub(1);

    for (index, (stage, redirects)) in stages.iter().enumerate() {
        let is_last = index == last_index;
        if let Some(result) = run_stage(stage, redirects, is_last, &mut ctx, &mut state)? {
            return Ok(result);
        }
    }

    state.drain_pending();
    let status = state.finish()?;
    Ok(CommandResult::Status(status))
}
