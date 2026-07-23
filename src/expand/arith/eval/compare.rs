//! Relational and equality operators (`<` `>` `<=` `>=` `==` `!=`).

use super::primary::skip_ws;
use super::shift::parse_shift;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn parse_compare(
    chars: &mut Peekable<Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_shift(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if eat_op2(chars, '=', '=') {
            left = i64::from(left == parse_shift(chars, env, last_status)?);
        } else if eat_op2(chars, '!', '=') {
            left = i64::from(left != parse_shift(chars, env, last_status)?);
        } else if eat_op2(chars, '<', '=') {
            left = i64::from(left <= parse_shift(chars, env, last_status)?);
        } else if eat_op2(chars, '>', '=') {
            left = i64::from(left >= parse_shift(chars, env, last_status)?);
        } else if chars.peek() == Some(&'<') && !peek_second(chars, '<') {
            chars.next();
            left = i64::from(left < parse_shift(chars, env, last_status)?);
        } else if chars.peek() == Some(&'>') && !peek_second(chars, '>') {
            chars.next();
            left = i64::from(left > parse_shift(chars, env, last_status)?);
        } else {
            return Ok(left);
        }
    }
}

fn eat_op2(chars: &mut Peekable<Chars<'_>>, a: char, b: char) -> bool {
    if chars.peek() != Some(&a) {
        return false;
    }
    let mut clone = chars.clone();
    clone.next();
    if clone.peek() != Some(&b) {
        return false;
    }
    chars.next();
    chars.next();
    true
}

fn peek_second(chars: &Peekable<Chars<'_>>, want: char) -> bool {
    let mut clone = chars.clone();
    clone.next();
    clone.peek() == Some(&want)
}
