//! Repeated Tab cycles through ambiguous matches after the first listing.

use super::complete_with_names;
use super::token::{apply_match, token_at};

/// Active menu-complete cycle after an ambiguous Tab listing.
#[derive(Debug, Clone)]
pub struct CompleteCycle {
    matches: Vec<String>,
    token_start: usize,
    expected: String,
    index: Option<usize>,
}

impl CompleteCycle {
    fn from_list(matches: Vec<String>, token_start: usize, stem: String) -> Self {
        Self {
            matches,
            token_start,
            expected: stem,
            index: None,
        }
    }

    fn still_valid(&self, buffer: &str, cursor: usize) -> bool {
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
        if active.still_valid(buffer, *cursor) {
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
