//! Running an external command as one stage of a `|` pipeline.

use super::state::{after_spawn_failure, AfterSpawnFail, PipeState};
use super::StageCtx;
use crate::exec::redirect::{apply_stdout_for_stage, RedirectFiles, StdinSource};
use crate::exec::{build_external_command, wait_children, CommandResult};

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
    name: &str,
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

    match spawn_and_feed(&mut command, stdin_bytes)? {
        Ok(mut child) => {
            state.prev_stdout = if !stdout_redirected && (!is_last || capturing) {
                child.stdout.take()
            } else {
                None
            };
            if is_last {
                return finish_last_external(child, ctx.stdout, state);
            }
            state.children.push(child);
            Ok(None)
        }
        Err(err) => match after_spawn_failure(name, &err, ctx.stderr, is_last, state)? {
            AfterSpawnFail::Continue => Ok(None),
            AfterSpawnFail::Done(result) => Ok(Some(result)),
        },
    }
}

fn finish_last_external(
    child: Child,
    stdout: &mut impl Write,
    state: &mut PipeState,
) -> io::Result<Option<CommandResult>> {
    if let Some(mut reader) = state.prev_stdout.take() {
        io::copy(&mut reader, stdout)?;
    }
    state.children.push(child);
    let status = wait_children(&mut state.children)?;
    Ok(Some(CommandResult::Status(status)))
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

fn spawn_and_feed(
    command: &mut Command,
    stdin_bytes: Option<Vec<u8>>,
) -> io::Result<Result<Child, io::Error>> {
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => return Ok(Err(err)),
    };
    if let Some(bytes) = stdin_bytes {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&bytes)?;
        }
    }
    Ok(Ok(child))
}
