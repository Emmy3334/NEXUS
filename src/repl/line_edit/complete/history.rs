//! History-word candidates for Tab completion.

use super::matchers::{matches_prefix, prefix_score};
use crate::history::History;

use std::collections::HashMap;

/// Collect unique history words matching `prefix`, with frecency boost scores.
pub(super) fn collect(history: &History, prefix: &str, out: &mut Vec<(String, i32)>) {
    let mut counts: HashMap<String, (u32, usize)> = HashMap::new();
    let total = history.len().max(1);
    for (i, (_, entry)) in history.iter().enumerate() {
        tally_line(&entry.line, prefix, i + 1, &mut counts);
    }
    for (word, (count, recency)) in counts {
        let freq = (count as i32).min(20);
        let recent = ((recency * 30) / total) as i32;
        out.push((word, freq + recent));
    }
}

fn tally_line(
    line: &str,
    prefix: &str,
    recency: usize,
    counts: &mut HashMap<String, (u32, usize)>,
) {
    for word in line.split_whitespace() {
        if !matches_prefix(word, prefix) {
            continue;
        }
        counts
            .entry(word.to_owned())
            .and_modify(|(c, r)| {
                *c += 1;
                *r = recency;
            })
            .or_insert((1, recency));
    }
}

/// Score for a history-only candidate (prefix match + frecency boost).
#[must_use]
pub(super) fn history_score(word: &str, prefix: &str, boost: i32) -> i32 {
    prefix_score(word, prefix).saturating_add(boost)
}
