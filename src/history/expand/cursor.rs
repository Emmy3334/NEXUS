//! Byte-index cursor over a UTF-8 string.

/// Peek the char at byte index `i`, if any.
#[must_use]
pub(super) fn peek(s: &str, i: usize) -> Option<char> {
    s.get(i..)?.chars().next()
}

/// Consume one char at `i`, advancing the byte index.
pub(super) fn bump(s: &str, i: &mut usize) -> Option<char> {
    let ch = peek(s, *i)?;
    *i += ch.len_utf8();
    Some(ch)
}
