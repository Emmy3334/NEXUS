//! Keep `${…}` spans inside a Word during lex (parens/operators would split).

use super::LexError;

/// If `open` is the `$` of a `${…}` expansion, return the index past the
/// matching `}`. Otherwise return `None`. Nested `${…}` are balanced.
pub(super) fn try_close(source: &str, open: usize) -> Result<Option<usize>, LexError> {
    let bytes = source.as_bytes();
    if bytes.get(open) != Some(&b'$') || bytes.get(open + 1) != Some(&b'{') {
        return Ok(None);
    }
    Ok(Some(scan_close(source, open + 1)?))
}

fn scan_close(source: &str, brace: usize) -> Result<usize, LexError> {
    let bytes = source.as_bytes();
    let mut depth = 0i32;
    let mut i = brace;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(LexError::UnclosedBraceParam)
}
