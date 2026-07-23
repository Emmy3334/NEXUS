//! Command substitution via `$(…)`.

use super::fields::FieldBuilder;
use super::subst_out;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

/// Consume `$(…)` (opening `$` already eaten; next char is `(`) and splice output.
pub(super) fn push_dollar_paren(
    chars: &mut Peekable<Chars<'_>>,
    fields: &mut FieldBuilder,
    split_words: bool,
    capture: &mut dyn FnMut(&str) -> Result<String, LexError>,
) -> Result<(), LexError> {
    chars.next(); // '('
    let body = take_body(chars)?;
    let output = capture(&body)?;
    subst_out::apply(fields, &output, split_words);
    Ok(())
}

/// Whether peek starts a `$(…)` (not `$((…))`).
#[must_use]
pub(super) fn looks_like(chars: &mut Peekable<Chars<'_>>) -> bool {
    if chars.peek() != Some(&'(') {
        return false;
    }
    let mut ahead = chars.clone();
    ahead.next();
    ahead.peek() != Some(&'(')
}

fn take_body(chars: &mut Peekable<Chars<'_>>) -> Result<String, LexError> {
    let mut body = String::new();
    let mut depth = 1i32;
    for ch in chars.by_ref() {
        match ch {
            '(' => {
                depth += 1;
                body.push(ch);
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(body);
                }
                body.push(ch);
            }
            _ => body.push(ch),
        }
    }
    Err(LexError::UnclosedCommandSubst)
}
