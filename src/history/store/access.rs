//! Lookups and iteration over stored history events.

use super::{History, HistoryEntry};

impl History {
    /// Event `n` (1-based), if present.
    #[must_use]
    pub fn get(&self, n: usize) -> Option<&str> {
        self.entry(n).map(|e| e.line.as_str())
    }

    /// Full entry `n` (1-based).
    #[must_use]
    pub(crate) fn entry(&self, n: usize) -> Option<&HistoryEntry> {
        if n == 0 {
            return None;
        }
        self.entries.get(n - 1)
    }

    /// Relative event: `1` is last, `2` is second-to-last, …
    #[must_use]
    pub fn get_relative(&self, n: usize) -> Option<&str> {
        self.entry_relative(n).map(|e| e.line.as_str())
    }

    /// Relative entry: `1` is last.
    #[must_use]
    pub(crate) fn entry_relative(&self, n: usize) -> Option<&HistoryEntry> {
        if n == 0 || n > self.entries.len() {
            return None;
        }
        self.entries.get(self.entries.len() - n)
    }

    /// Most recent event whose text starts with `prefix`.
    #[must_use]
    pub fn find_prefix(&self, prefix: &str) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.line.starts_with(prefix))
            .map(|e| e.line.as_str())
    }

    /// Most recent event containing `needle` anywhere.
    #[must_use]
    pub fn find_substring(&self, needle: &str) -> Option<&str> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.line.contains(needle))
            .map(|e| e.line.as_str())
    }

    /// Iterate `(1-based number, entry)` oldest-first.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (usize, &HistoryEntry)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(i, entry)| (i + 1, entry))
    }
}
