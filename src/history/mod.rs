//! In-memory command history and `!` event expansion.

mod expand;
mod path;
mod store;

pub use expand::{expand_line, ExpandOutcome, HistoryError};
pub use path::resolve as histfile_path;
pub use store::{History, HistoryEntry};
