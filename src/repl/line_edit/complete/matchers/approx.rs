//! Levenshtein distance ≤1 fallback matching.

/// Edit distance with early exit when distance would exceed `max`.
#[must_use]
pub fn edit_distance(a: &str, b: &str, max: usize) -> usize {
    let (a, b) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if b.len().saturating_sub(a.len()) > max {
        return max + 1;
    }
    let mut prev: Vec<usize> = (0..=a.len()).collect();
    for (i, bc) in b.chars().enumerate() {
        let mut row = vec![i + 1];
        let mut row_min = row[0];
        for (j, ac) in a.chars().enumerate() {
            let cost = usize::from(ac != bc);
            let v = (prev[j + 1] + 1).min(row[j] + 1).min(prev[j] + cost);
            row.push(v);
            row_min = row_min.min(v);
        }
        if row_min > max {
            return max + 1;
        }
        prev = row;
    }
    prev[a.len()]
}

/// Candidates within edit distance ≤1 of `prefix`, nearest first.
pub fn approx_matches(candidates: &[String], prefix: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for c in candidates {
        let d = edit_distance(c, prefix, 1);
        if d <= 1 {
            out.push((c.clone(), d));
        }
    }
    out.sort_by_key(|(_, d)| *d);
    out
}
