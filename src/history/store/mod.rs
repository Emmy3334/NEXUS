//! Stored command lines for event designators and the `history` builtin.

mod access;
mod file;

use std::time::SystemTime;

/// Default max events retained when `histsize` is unset.
pub const DEFAULT_HISTSIZE: usize = 10_000;

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
        self.push_limited(line, DEFAULT_HISTSIZE);
    }

    /// Like [`push`], respecting an explicit `histsize` cap.
    pub fn push_limited(&mut self, line: impl Into<String>, limit: usize) {
        let line = line.into();
        if line.is_empty() {
            return;
        }
        self.entries.push(HistoryEntry {
            line,
            time: SystemTime::now(),
        });
        self.trim_to(limit);
    }

    /// Append with an explicit timestamp (histfile load / tests).
    pub fn push_at(&mut self, line: impl Into<String>, time: SystemTime) {
        let line = line.into();
        if line.is_empty() {
            return;
        }
        self.entries.push(HistoryEntry { line, time });
        self.trim_to(DEFAULT_HISTSIZE);
    }

    /// Remove all events.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_subst = None;
        self.last_search_word = None;
    }

    fn trim_to(&mut self, limit: usize) {
        if limit == 0 {
            self.entries.clear();
            return;
        }
        if self.entries.len() > limit {
            let drop_n = self.entries.len() - limit;
            self.entries.drain(..drop_n);
        }
    }
}
