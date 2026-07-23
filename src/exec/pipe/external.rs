//! Running an external command as one stage of a `|` pipeline.

use super::state::PipeState;
use super::StageCtx;
use crate::exec::redirect::{apply_stdout_for_stage, RedirectFiles, StdinSource};
use crate::exec::{build_external_command, CommandResult};
use crate::heal;

use std::fs::File;
use std::io::{self, BufRead, Write};
use std::process::{Child, ChildStdout, Command, Stdio};

/// Where this stage's stdin comes from, resolved from the redirect files and
/// prior-stage state.
enum StageStdin {
    FromFile(File),
    Bytes(Vec<u8>),
    FromPipe(ChildStdout),
    None,
}

/// Run an external stage. Returns `Some(result)` when the pipeline should
/// stop (spawn failure on the last stage); `None` to continue.
pub(super) fn run_external_stage<I: BufRead, O: Write, E: Write>(
    stage: &[String],
    files: RedirectFiles,
    is_last: bool,
    ctx: &mut StageCtx<'_, I, O, E>,
    state: &mut PipeState,
) -> io::Result<Option<CommandResult>> {
    let mut command = build_external_command(stage, ctx.shell_env);
    let stdout_redirected = files.stdout.is_some();
    let capturing = ctx.stdout_mode == crate::exec::StdoutMode::Capture;
    let stdin = resolve_stage_stdin(files.stdin, state);
    let stdin_bytes = apply_stage_stdin(&mut command, stdin);
    apply_stdout_for_stage(&mut command, files.stdout, is_last, ctx.stdout_mode);
    state.prepare_command(&mut command);
    match spawn_and_feed(&mut command, stdin_bytes)? {
        SpawnFeed::Child(child) => {
            take_spawned(child, stdout_redirected, capturing, is_last, ctx, state)
        }
        SpawnFeed::Failed { err, stdin } => {
            note_spawn_failure(&err, is_last, ctx, state, stage, stdin.as_deref())
        }
    }
}

fn take_spawned<I: BufRead, O: Write, E: Write>(
    mut child: Child,
    stdout_redirected: bool,
    capturing: bool,
    is_last: bool,
    ctx: &mut StageCtx<'_, I, O, E>,
    state: &mut PipeState,
) -> io::Result<Option<CommandResult>> {
    state.prev_stdout = if !stdout_redirected && (!is_last || capturing) {
        child.stdout.take()
    } else {
        None
    };
    if is_last {
        return finish_last_external(child, ctx.stdout, state);
    }
    state.push_child(child);
    Ok(None)
}

fn note_spawn_failure<I: BufRead, O: Write, E: Write>(
    err: &io::Error,
    is_last: bool,
    ctx: &mut StageCtx<'_, I, O, E>,
    state: &mut PipeState,
    stage: &[String],
    stdin: Option<&[u8]>,
) -> io::Result<Option<CommandResult>> {
    let code =
        heal::after_spawn_failure_stdin(stage, err, ctx.shell_env, stdin, ctx.stdout, ctx.stderr)?;
    state.drain_pending();
    if is_last {
        state.terminal_status = Some(code);
    }
    Ok(None)
}

fn finish_last_external(
    child: Child,
    stdout: &mut impl Write,
    state: &mut PipeState,
) -> io::Result<Option<CommandResult>> {
    if let Some(mut reader) = state.prev_stdout.take() {
        io::copy(&mut reader, stdout)?;
    }
    state.push_child(child);
    Ok(None)
}

fn resolve_stage_stdin(files_stdin: Option<StdinSource>, state: &mut PipeState) -> StageStdin {
    match files_stdin {
        Some(StdinSource::File(file)) => {
            state.drain_pending();
            StageStdin::FromFile(file)
        }
        Some(StdinSource::Bytes(bytes)) => {
            state.drain_pending();
            StageStdin::Bytes(bytes)
        }
        None => {
            if let Some(buffer) = state.buffered_out.take() {
                StageStdin::Bytes(buffer)
            } else if let Some(pipe) = state.prev_stdout.take() {
                StageStdin::FromPipe(pipe)
            } else {
                StageStdin::None
            }
        }
    }
}

fn apply_stage_stdin(command: &mut Command, stdin: StageStdin) -> Option<Vec<u8>> {
    match stdin {
        StageStdin::FromFile(file) => {
            command.stdin(Stdio::from(file));
            None
        }
        StageStdin::Bytes(bytes) => {
            command.stdin(Stdio::piped());
            Some(bytes)
        }
        StageStdin::FromPipe(pipe) => {
            command.stdin(pipe);
            None
        }
        StageStdin::None => None,
    }
}

fn spawn_and_feed(command: &mut Command, stdin_bytes: Option<Vec<u8>>) -> io::Result<SpawnFeed> {
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            return Ok(SpawnFeed::Failed {
                err,
                stdin: stdin_bytes,
            })
        }
    };
    if let Some(bytes) = stdin_bytes {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&bytes)?;
        }
    }
    Ok(SpawnFeed::Child(child))
}

enum SpawnFeed {
    Child(Child),
    Failed {
        err: io::Error,
        stdin: Option<Vec<u8>>,
    },
}
