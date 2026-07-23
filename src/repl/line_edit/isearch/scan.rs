//! Scan history for the next reverse-i-search hit.

use super::HistoryISearch;

pub(super) fn research(search: &mut HistoryISearch<'_>, from: usize) {
    let len = search.history.len();
    if len == 0 {
        search.match_offset = 0;
        search.failed = true;
        return;
    }
    let start = from.max(1);
    for n in start..=len {
        let Some(line) = search.history.get_relative(n) else {
            continue;
        };
        if search.query.is_empty() || line.contains(search.query.as_str()) {
            search.match_offset = n;
            search.failed = false;
            return;
        }
    }
    search.failed = true;
}
