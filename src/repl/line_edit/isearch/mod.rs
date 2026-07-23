//! Incremental reverse history search (Ctrl-R).

mod scan;

use crate::history::History;

/// Active `(reverse-i-search)` session.
#[derive(Debug)]
pub struct HistoryISearch<'a> {
    history: &'a History,
    pub query: String,
    /// Relative history index of the current match (`1` = newest); `0` = none.
    match_offset: usize,
    pub failed: bool,
    draft: String,
}

impl<'a> HistoryISearch<'a> {
    /// Begin search; `draft` is restored on abort.
    #[must_use]
    pub fn start(history: &'a History, draft: impl Into<String>) -> Self {
        let mut search = Self {
            history,
            query: String::new(),
            match_offset: 0,
            failed: false,
            draft: draft.into(),
        };
        scan::research(&mut search, 1);
        search
    }

    #[must_use]
    pub fn draft(&self) -> &str {
        &self.draft
    }

    /// Line to show in the edit buffer (match, or draft when failed).
    #[must_use]
    pub fn display_line(&self) -> &str {
        if self.failed {
            return self.draft.as_str();
        }
        self.history
            .get_relative(self.match_offset)
            .unwrap_or(self.draft.as_str())
    }

    /// Prompt prefix including the query.
    #[must_use]
    pub fn prompt_label(&self) -> String {
        let tag = if self.failed {
            "failed r-search"
        } else {
            "reverse-i-search"
        };
        format!("({tag})`{}': ", self.query)
    }

    pub fn push_char(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }
        self.query.push(ch);
        scan::research(self, 1);
    }

    pub fn backspace(&mut self) {
        if self.query.pop().is_some() {
            scan::research(self, 1);
        }
    }

    /// Find an older match (another Ctrl-R).
    pub fn again(&mut self) {
        let from = if self.match_offset == 0 {
            1
        } else {
            self.match_offset.saturating_add(1)
        };
        scan::research(self, from);
    }
}
