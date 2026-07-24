//! Comma operator — lowest precedence; value is the rightmost expression.

use super::assign::parse_assign;
use super::primary::skip_ws;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

/// `expr ( ',' expr )*` — evaluates left-to-right, yields the last value.
pub(super) fn parse_comma(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut value = parse_assign(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if chars.peek() != Some(&',') {
            return Ok(value);
        }
        chars.next();
        value = parse_assign(chars, env, last_status)?;
    }
}
