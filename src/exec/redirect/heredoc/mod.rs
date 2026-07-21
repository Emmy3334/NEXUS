//! Heredoc (`<<`) body collection and lookup.

mod collect;

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
}

pub use collect::collect_heredoc_bodies;
