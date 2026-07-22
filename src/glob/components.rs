//! Split a glob pattern on `/` into path components.

pub(super) fn split_pattern_components(pattern: &[(char, bool)]) -> Vec<Vec<(char, bool)>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    for &(c, meta) in pattern {
        if c == '/' && !meta {
            if !current.is_empty() {
                parts.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push((c, meta));
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}
