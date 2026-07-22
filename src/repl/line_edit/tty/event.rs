//! Key events produced by the raw-mode reader.

use crate::repl::line_edit::bindings::Action;

#[derive(Debug)]
pub(super) enum Event {
    Action(Action),
    /// One or more printable characters (paste-friendly).
    InsertRun(String),
    Raw(Vec<u8>),
}
