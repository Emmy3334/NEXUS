//! Keep `[[ … ]]` conditional commands inside one Word during lex.

use super::{quote, LexError, QuoteState};

/// If `open` starts `[[`, return the index past the matching `]]`.
pub(super) fn try_close(source: &str, open: usize) -> Result<Option<usize>, LexError> {
    let bytes = source.as_bytes();
    if bytes.get(open) != Some(&b'[') || bytes.get(open + 1) != Some(&b'[') {
        return Ok(None);
    }
    let mut state = QuoteState::Normal;
    let mut i = open + 2;
    while i < bytes.len() {
        if state == QuoteState::Normal
            && bytes.get(i) == Some(&b']')
            && bytes.get(i + 1) == Some(&b']')
        {
            return Ok(Some(i + 2));
        }
        let ch = source[i..].chars().next().expect("i is a char boundary");
        if quote::escapes(state) && ch == '\\' {
            i += ch.len_utf8();
            if let Some(next) = source[i..].chars().next() {
                i += next.len_utf8();
            }
            continue;
        }
        state = quote::advance_at(state, source, i);
        i += ch.len_utf8();
    }
    Err(LexError::UnclosedConditional)
}
