//! Parse and match a `[…]` character class in a glob pattern.

/// Parse `[…]` at `pi`. Returns `(index_after_class, character_matched)`.
pub(super) fn match_bracket(
    pattern: &[(char, bool)],
    pi: usize,
    ch: char,
) -> Option<(usize, bool)> {
    let mut i = pi + 1;
    if i >= pattern.len() {
        return None;
    }
    let mut negated = false;
    if matches!(pattern[i].0, '!' | '^') {
        negated = true;
        i += 1;
    }
    match_bracket_body(pattern, i, ch, negated)
}

fn match_bracket_body(
    pattern: &[(char, bool)],
    mut i: usize,
    ch: char,
    negated: bool,
) -> Option<(usize, bool)> {
    let mut matched = false;
    let mut first = true;
    while i < pattern.len() {
        let c = pattern[i].0;
        if c == ']' && !first {
            let ok = if negated { !matched } else { matched };
            return Some((i + 1, ok));
        }
        first = false;
        i = consume_class_atom(pattern, i, ch, &mut matched);
    }
    None
}

fn consume_class_atom(pattern: &[(char, bool)], i: usize, ch: char, matched: &mut bool) -> usize {
    let c = pattern[i].0;
    if i + 2 < pattern.len() && pattern[i + 1].0 == '-' && pattern[i + 2].0 != ']' {
        let start = c;
        let end = pattern[i + 2].0;
        if start <= ch && ch <= end {
            *matched = true;
        }
        return i + 3;
    }
    if c == ch {
        *matched = true;
    }
    i + 1
}
