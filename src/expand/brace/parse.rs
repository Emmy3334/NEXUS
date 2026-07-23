//! Parse `{alt,alt,…}` alternatives starting at `{`.

/// Returns alternatives and the index past the closing `}`, or `None` if not expandable.
pub(super) fn alternatives(chars: &[char], open: usize) -> Option<(Vec<String>, usize)> {
    if chars.get(open) != Some(&'{') {
        return None;
    }
    let mut depth = 1usize;
    let mut start = open + 1;
    let mut alts = Vec::new();
    let mut saw_comma = false;
    let mut i = open + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                i = i.saturating_add(2);
                continue;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    alts.push(chars[start..i].iter().collect());
                    return if saw_comma { Some((alts, i + 1)) } else { None };
                }
            }
            ',' if depth == 1 => {
                saw_comma = true;
                alts.push(chars[start..i].iter().collect());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    None
}
