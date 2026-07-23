//! Mutable state threaded through a multi-stage pipeline run.

use crate::exec::wait_children;
use crate::jobs::{self, FgWait};

use std::io;
use std::process::{Child, ChildStdout, Command};

/// Children spawned so far, the previous stage's readable stdout (when piped),
/// and a builtin's buffered stdout (when the next stage needs to consume it).
pub(super) struct PipeState {
    pub(super) children: Vec<Child>,
    pub(super) prev_stdout: Option<ChildStdout>,
    pub(super) buffered_out: Option<Vec<u8>>,
    pub(super) terminal_status: Option<u8>,
    pgid: Option<i32>,
}

impl PipeState {
    pub(super) fn new() -> Self {
        Self {
            children: Vec::new(),
            prev_stdout: None,
            buffered_out: None,
            terminal_status: None,
            pgid: None,
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

    pub(super) fn prepare_command(
        &self,
        command: &mut Command,
        env: &crate::env::ShellEnvironment,
    ) {
        jobs::prepare_process_group(command, self.pgid, env);
    }

    pub(super) fn push_child(&mut self, child: Child) {
        if jobs::job_control_enabled() {
            self.pgid = Some(jobs::assign_process_group(child.id(), self.pgid));
        }
        self.children.push(child);
    }

    pub(super) fn finish(&mut self) -> io::Result<FgWait> {
        let Some(pgid) = self.pgid else {
            let status = wait_children(&mut self.children)?;
            return Ok(FgWait::Done(self.terminal_status.unwrap_or(status)));
        };
        let outcome = jobs::wait_pipeline(&mut self.children, pgid)?;
        if let FgWait::Done(status) = outcome {
            for child in &mut self.children {
                let _ = child.try_wait();
            }
            self.children.clear();
            return Ok(FgWait::Done(self.terminal_status.unwrap_or(status)));
        }
        Ok(outcome)
    }
}
