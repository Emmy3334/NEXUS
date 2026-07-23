//! Read `$((` body until the matching `))`.

use crate::lex::LexError;

/// `chars` positioned after both opening `(`.
pub(super) fn take_body(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<String, LexError> {
    let mut body = String::new();
    let mut depth = 0i32;
    while let Some(ch) = chars.next() {
        match ch {
            '(' => {
                depth += 1;
                body.push(ch);
            }
            ')' => {
                if depth == 0 {
                    if chars.peek() == Some(&')') {
                        chars.next();
                        return Ok(body);
                    }
                    return Err(LexError::UnclosedArithmetic);
                }
                depth -= 1;
                body.push(ch);
            }
            _ => body.push(ch),
        }
    }
    Err(LexError::UnclosedArithmetic)
}
