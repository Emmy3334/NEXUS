//! Rank candidate command names against a mistyped program.

const MAX_SUGGESTIONS: usize = 3;

/// Best prefix / edit-distance matches for `needle` (capped).
#[must_use]
pub(super) fn top(needle: &str, candidates: &[String]) -> Vec<String> {
    let mut scored: Vec<(usize, &str)> = candidates
        .iter()
        .filter_map(|c| {
            if c == needle {
                return None;
            }
            let dist = score(needle, c)?;
            Some((dist, c.as_str()))
        })
        .collect();
    scored.sort_by_key(|(d, name)| (*d, name.len(), *name));
    scored
        .into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(_, name)| name.to_owned())
        .collect()
}

fn score(needle: &str, candidate: &str) -> Option<usize> {
    if candidate.starts_with(needle) || needle.starts_with(candidate) {
        return Some(0);
    }
    let dist = edit_distance(needle, candidate);
    let max = needle.len().max(2) / 2 + 1;
    (dist <= max).then_some(dist)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut cur = vec![0; n + 1];
    for i in 1..=m {
        cur[0] = i;
        for j in 1..=n {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[n]
}
