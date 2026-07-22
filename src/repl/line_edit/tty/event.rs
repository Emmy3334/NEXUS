//! Key events produced by the raw-mode reader.

use crate::keybind::Action;

#[derive(Debug)]
pub(super) enum Event {
    Action(Action),
    /// One or more printable characters (paste-friendly).
    InsertRun(String),
    Raw(Vec<u8>),
    /// OSC-style bracketed paste begin (`\e[200~`).
    PasteStart,
    /// Bracketed paste end (`\e[201~`).
    PasteEnd,
}
