//! In-memory command history and `!` event expansion.

mod expand;
mod store;

pub use expand::{expand_line, ExpandOutcome, HistoryError};
pub use store::{History, HistoryEntry};
