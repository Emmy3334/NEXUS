//! Command substitution via `` `…` ``.

use super::fields::FieldBuilder;
use super::subst_out;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

/// Consume a backtick body (opening `` ` `` already eaten) and splice output.
pub(super) fn push_backtick(
    chars: &mut Peekable<Chars<'_>>,
    fields: &mut FieldBuilder,
    split_words: bool,
    capture: &mut dyn FnMut(&str) -> Result<String, LexError>,
) -> Result<(), LexError> {
    let body = take_body(chars)?;
    let output = capture(&body)?;
    subst_out::apply(fields, &output, split_words);
    Ok(())
}

fn take_body(chars: &mut Peekable<Chars<'_>>) -> Result<String, LexError> {
    let mut body = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '`' => return Ok(body),
            '\\' => {
                if let Some(next) = chars.next() {
                    body.push(next);
                }
            }
            _ => body.push(ch),
        }
    }
    Err(LexError::UnclosedQuote)
}
