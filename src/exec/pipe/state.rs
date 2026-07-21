//! Mutable state threaded through a multi-stage pipeline run.

use crate::exec::{report_spawn_failure, wait_children, CommandResult};

use std::io::{self, Write};
use std::process::{Child, ChildStdout};

/// Children spawned so far, the previous stage's readable stdout (when piped),
/// and a builtin's buffered stdout (when the next stage needs to consume it).
pub(super) struct PipeState {
    pub(super) children: Vec<Child>,
    pub(super) prev_stdout: Option<ChildStdout>,
    pub(super) buffered_out: Option<Vec<u8>>,
}

impl PipeState {
    pub(super) fn new() -> Self {
        Self {
            children: Vec::new(),
            prev_stdout: None,
            buffered_out: None,
        }
    }

    /// Drain (rather than leave hanging) any prior stage's output that this
    /// stage will not consume, so upstream writers can exit.
    pub(super) fn drain_pending(&mut self) {
        if let Some(mut reader) = self.prev_stdout.take() {
            let _ = io::copy(&mut reader, &mut io::sink());
        }
        let _ = self.buffered_out.take();
    }

    pub(super) fn finish(&mut self) -> io::Result<u8> {
        wait_children(&mut self.children)
    }
}

/// What to do after a pipeline stage fails to spawn.
pub(super) enum AfterSpawnFail {
    /// Skip this stage; later stages still run (no pipefail).
    Continue,
    /// This was the last stage — pipeline status is the spawn failure code.
    Done(CommandResult),
}

/// Report spawn failure, drain unused pipe input, then continue or finish.
pub(super) fn after_spawn_failure(
    name: &str,
    err: &io::Error,
    stderr: &mut impl Write,
    is_last: bool,
    state: &mut PipeState,
) -> io::Result<AfterSpawnFail> {
    let code = report_spawn_failure(name, err, stderr)?;
    state.drain_pending();
    if is_last {
        let _ = state.finish()?;
        return Ok(AfterSpawnFail::Done(CommandResult::Status(code)));
    }
    Ok(AfterSpawnFail::Continue)
}
