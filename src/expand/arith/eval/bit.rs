//! Bitwise `|`, `^`, `&`.

use super::compare::parse_compare;
use super::primary::skip_ws;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn parse_bitor(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_bitxor(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if chars.peek() == Some(&'|') && !peek_second(chars, '|') {
            chars.next();
            left |= parse_bitxor(chars, env, last_status)?;
        } else {
            return Ok(left);
        }
    }
}

fn parse_bitxor(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_bitand(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if chars.peek() == Some(&'^') {
            chars.next();
            left ^= parse_bitand(chars, env, last_status)?;
        } else {
            return Ok(left);
        }
    }
}

fn parse_bitand(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_compare(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if chars.peek() == Some(&'&') && !peek_second(chars, '&') {
            chars.next();
            left &= parse_compare(chars, env, last_status)?;
        } else {
            return Ok(left);
        }
    }
}

fn peek_second(chars: &Peekable<Chars<'_>>, want: char) -> bool {
    let mut clone = chars.clone();
    clone.next();
    clone.peek() == Some(&want)
}
