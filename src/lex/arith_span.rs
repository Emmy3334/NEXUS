//! Keep `$((…))` spans inside a Word during lex (parens would otherwise split).

use super::LexError;

/// If `open` is the first `(` of `$ ((`, return the byte index past the matching
/// closing `))` (expression parentheses nest). Otherwise return `None`.
pub(super) fn try_close_arith(source: &str, open: usize) -> Result<Option<usize>, LexError> {
    let bytes = source.as_bytes();
    if open == 0 || bytes.get(open) != Some(&b'(') || bytes.get(open - 1) != Some(&b'$') {
        return Ok(None);
    }
    if bytes.get(open + 1) != Some(&b'(') {
        return Ok(None);
    }
    Ok(Some(scan_close(source, open)?))
}

fn scan_close(source: &str, open: usize) -> Result<usize, LexError> {
    let bytes = source.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(LexError::UnclosedArithmetic)
}
