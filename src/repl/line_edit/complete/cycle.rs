//! Repeated Tab cycles through ambiguous matches after the first listing.

use super::complete_with_names;
use super::token::{apply_match, token_at};

/// Active menu-complete cycle after an ambiguous Tab listing.
#[derive(Debug, Clone)]
pub struct CompleteCycle {
    pub matches: Vec<String>,
    pub highlight: usize,
    /// How many TTY rows the menu occupies (for rewrite on Up/Down).
    pub list_rows: usize,
    token_start: usize,
    expected: String,
    index: Option<usize>,
}

impl CompleteCycle {
    fn from_list(matches: Vec<String>, token_start: usize, stem: String) -> Self {
        Self {
            matches,
            highlight: 0,
            list_rows: 0,
            token_start,
            expected: stem,
            index: None,
        }
    }

    /// True while the token under the cursor still matches this menu.
    #[must_use]
    pub fn is_active(&self, buffer: &str, cursor: usize) -> bool {
        let (start, token) = token_at(buffer, cursor);
        start == self.token_start && token == self.expected
    }

    fn advance(&mut self, buffer: &mut String, cursor: &mut usize) {
        let next = match self.index {
            None => 0,
            Some(i) => (i + 1) % self.matches.len(),
        };
        let value = self.matches[next].clone();
        apply_match(buffer, cursor, self.token_start, &value);
        self.expected = value;
        self.index = Some(next);
        self.highlight = next;
    }

    /// Move the menu highlight up (wrap).
    pub fn move_up(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.highlight = if self.highlight == 0 {
            self.matches.len() - 1
        } else {
            self.highlight - 1
        };
    }

    /// Move the menu highlight down (wrap).
    pub fn move_down(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.highlight = (self.highlight + 1) % self.matches.len();
    }

    /// Insert the highlighted match (does not submit the line).
    pub fn accept(&self, buffer: &mut String, cursor: &mut usize) {
        if let Some(value) = self.matches.get(self.highlight) {
            apply_match(buffer, cursor, self.token_start, value);
        }
    }
}

/// Like [`super::complete_with_names`], then cycle matches on further Tabs.
pub fn complete_or_cycle(
    buffer: &mut String,
    cursor: &mut usize,
    var_names: &[String],
    cycle: &mut Option<CompleteCycle>,
) -> Vec<String> {
    if let Some(active) = cycle.as_mut() {
        if active.is_active(buffer, *cursor) {
            active.advance(buffer, cursor);
            return Vec::new();
        }
    }
    *cycle = None;
    let listed = complete_with_names(buffer, cursor, var_names);
    if listed.len() > 1 {
        let (start, stem) = token_at(buffer, *cursor);
        *cycle = Some(CompleteCycle::from_list(listed.clone(), start, stem));
    }
    listed
}
