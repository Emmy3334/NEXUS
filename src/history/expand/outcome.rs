//! History expansion result and error types.

/// Failure during history expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryError {
    /// No matching event for this designator.
    EventNotFound,
    /// Bad word designator or modifier.
    BadModifier,
}

impl HistoryError {
    #[must_use]
    pub fn message(&self) -> &'static str {
        match self {
            Self::EventNotFound => "Event not found.",
            Self::BadModifier => "Bad ! modifier.",
        }
    }
}

/// Result of expanding history on a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpandOutcome {
    /// Whether any `!` / `^` substitution occurred.
    pub changed: bool,
    /// `:p` — print the line but do not execute.
    pub print_only: bool,
}
