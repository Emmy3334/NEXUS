//! Browse session history (↑/↓) while composing a line.
//!
//! Lives outside the TTY module so integration tests can cover recall without
//! raw-mode stdin.

use crate::history::History;

/// Cursor into prior commands; `offset == 0` is the live draft.
#[derive(Debug)]
pub struct HistoryRecall<'a> {
    history: &'a History,
    offset: usize,
    draft: String,
}

impl<'a> HistoryRecall<'a> {
    #[must_use]
    pub fn new(history: &'a History) -> Self {
        Self {
            history,
            offset: 0,
            draft: String::new(),
        }
    }

    /// How many steps back from the draft (`0` = editing the draft).
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Move to an older event; no-op at the oldest entry.
    pub fn older(&mut self, line: &mut String) {
        let next = self.offset.saturating_add(1);
        let Some(text) = self.history.get_relative(next) else {
            return;
        };
        if self.offset == 0 {
            self.draft.clone_from(line);
        }
        self.offset = next;
        line.clear();
        line.push_str(text);
    }

    /// Move toward the draft; no-op when already on the draft.
    pub fn newer(&mut self, line: &mut String) {
        if self.offset == 0 {
            return;
        }
        self.offset -= 1;
        if self.offset == 0 {
            line.clear();
            line.push_str(&self.draft);
        } else if let Some(text) = self.history.get_relative(self.offset) {
            line.clear();
            line.push_str(text);
        }
    }
}
