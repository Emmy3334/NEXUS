//! Ternary `cond ? then : else`.

use super::logic::parse_or;
use super::primary::skip_ws;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn parse_ternary(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let cond = parse_or(chars, env, last_status)?;
    skip_ws(chars);
    if chars.peek() != Some(&'?') {
        return Ok(cond);
    }
    chars.next();
    let then_v = parse_ternary(chars, env, last_status)?;
    skip_ws(chars);
    if chars.next() != Some(':') {
        return Err(LexError::Arithmetic);
    }
    let else_v = parse_ternary(chars, env, last_status)?;
    Ok(if cond != 0 { then_v } else { else_v })
}
