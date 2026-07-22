//! Stored command lines for event designators and the `history` builtin.

mod access;
mod file;

use std::time::SystemTime;

/// One history event with wall-clock stamp (for `-T` / `-M`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub line: String,
    pub time: SystemTime,
}

/// Session command history (1-based event numbers).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    pub(super) entries: Vec<HistoryEntry>,
    /// Last `:s/l/r/` pair for `:&` / empty left-hand `:s`.
    pub(super) last_subst: Option<(String, String)>,
    /// Word matched by the most recent `!?s?` search (`:%`).
    pub(super) last_search_word: Option<String>,
}

impl History {
    /// Number of stored events.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether history is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Append a command line (after history expansion), stamped now.
    pub fn push(&mut self, line: impl Into<String>) {
        let line = line.into();
        if !line.is_empty() {
            self.entries.push(HistoryEntry {
                line,
                time: SystemTime::now(),
            });
        }
    }

    /// Append with an explicit timestamp (histfile load / tests).
    pub fn push_at(&mut self, line: impl Into<String>, time: SystemTime) {
        let line = line.into();
        if !line.is_empty() {
            self.entries.push(HistoryEntry { line, time });
        }
    }

    /// Remove all events.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_subst = None;
        self.last_search_word = None;
    }
}
