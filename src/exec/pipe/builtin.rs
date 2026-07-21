//! Running a builtin as one stage of a `|` pipeline (subshell semantics).

use super::state::PipeState;
use super::StageCtx;
use crate::env::ShellEnvironment;
use crate::exec::redirect::RedirectFiles;
use crate::exec::{run_builtin_status, CommandResult};

use std::fs::File;
use std::io::{self, BufRead, Write};

/// Run a builtin stage. Returns `Some(result)` when the pipeline should stop
/// (this was the last stage); `None` to continue to the next stage.
pub(super) fn run_builtin_stage<I: BufRead, O: Write, E: Write>(
    stage: &[String],
    files: RedirectFiles,
    is_last: bool,
    ctx: &mut StageCtx<'_, I, O, E>,
    state: &mut PipeState,
) -> io::Result<Option<CommandResult>> {
    state.drain_pending();
    // Builtins don't consume stdin yet; drop any stdin redirect.
    drop(files.stdin);

    let mut env_clone = ctx.shell_env.clone();
    if is_last {
        let status = run_builtin_to(
            stage,
            files.stdout,
            &mut env_clone,
            ctx.last_status,
            ctx.stdout,
            ctx.stderr,
        )?;
        state.terminal_status = Some(status);
        return Ok(None);
    }

    buffer_builtin_output(
        stage,
        files.stdout,
        &mut env_clone,
        ctx.last_status,
        ctx.stderr,
        state,
    )?;
    Ok(None)
}

/// Not the last stage: run the builtin, buffering its stdout (unless
/// redirected to a file) so the next external stage can consume it.
fn buffer_builtin_output<E: Write>(
    stage: &[String],
    stdout_file: Option<File>,
    env_clone: &mut ShellEnvironment,
    last_status: u8,
    stderr: &mut E,
    state: &mut PipeState,
) -> io::Result<()> {
    match stdout_file {
        Some(mut file) => {
            let _ = run_builtin_status(stage, env_clone, last_status, &mut file, stderr)?;
        }
        None => {
            let mut buffer = Vec::new();
            let _ = run_builtin_status(stage, env_clone, last_status, &mut buffer, stderr)?;
            state.buffered_out = Some(buffer);
        }
    }
    Ok(())
}

/// Run a builtin to `stdout_file` when set, otherwise to the pipeline's own stdout.
fn run_builtin_to<O: Write, E: Write>(
    stage: &[String],
    stdout_file: Option<File>,
    env_clone: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut O,
    stderr: &mut E,
) -> io::Result<u8> {
    match stdout_file {
        Some(mut file) => run_builtin_status(stage, env_clone, last_status, &mut file, stderr),
        None => run_builtin_status(stage, env_clone, last_status, stdout, stderr),
    }
}
