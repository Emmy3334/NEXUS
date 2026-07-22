//! Heredoc (`<<`) body collection and lookup.

mod collect;

use crate::parse::{Pipeline, PipelineCommand, Redirect, RedirectKind};

use std::io::Write;

/// Cursor over pre-collected heredoc bodies (one per `<<`, left-to-right).
///
/// Takes ownership of each body when applied so we do not clone the payload.
pub(in crate::exec) struct HeredocState {
    bodies: Vec<String>,
    index: usize,
}

impl HeredocState {
    pub(in crate::exec) fn new(bodies: Vec<String>) -> Self {
        Self { bodies, index: 0 }
    }

    pub(in crate::exec) fn take_next(
        &mut self,
        stderr: &mut impl Write,
    ) -> std::io::Result<Result<String, u8>> {
        if self.index >= self.bodies.len() {
            writeln!(stderr, "nexus: missing heredoc body")?;
            return Ok(Err(1));
        }
        let body = std::mem::take(&mut self.bodies[self.index]);
        self.index += 1;
        Ok(Ok(body))
    }

    /// Advance past bodies for a short-circuited pipeline (same order as collect).
    pub(in crate::exec) fn discard_pipeline(&mut self, pipeline: &Pipeline<'_>) {
        for command in &pipeline.commands {
            self.discard_command(command);
        }
    }

    fn discard_command(&mut self, command: &PipelineCommand<'_>) {
        match command {
            PipelineCommand::Simple(simple) => self.discard_redirects(&simple.redirects),
            PipelineCommand::Subshell { list, redirects } => {
                self.discard_redirects(redirects);
                for nested in &list.pipelines {
                    self.discard_pipeline(nested);
                }
            }
        }
    }

    fn discard_redirects(&mut self, redirects: &[Redirect<'_>]) {
        for redirect in redirects {
            if redirect.kind == RedirectKind::Heredoc && self.index < self.bodies.len() {
                self.index += 1;
            }
        }
    }
}

pub use collect::collect_heredoc_bodies;
