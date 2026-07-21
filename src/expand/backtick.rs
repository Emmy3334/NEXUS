//! Command substitution via `` `…` ``.

use super::fields::FieldBuilder;
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
    if split_words {
        splice_split(fields, &output);
    } else {
        let joined = output.replace(['\n', '\r'], " ");
        fields.current().push_str_literal(&joined);
    }
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

fn splice_split(fields: &mut FieldBuilder, output: &str) {
    let pieces: Vec<&str> = output.split_whitespace().collect();
    if pieces.is_empty() {
        return;
    }
    fields.current().push_str_globable(pieces[0]);
    for piece in &pieces[1..] {
        fields.start_field();
        fields.current().push_str_globable(piece);
    }
}
