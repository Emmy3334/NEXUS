//! Expand-stage completion (stub for PR1).

use super::super::match_item::Match;

/// Post-process matches (no-op until expand matchers land).
#[must_use]
pub fn expand(_before: &str, matches: Vec<Match>) -> Vec<Match> {
    matches
}
