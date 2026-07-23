//! Logical `||` and `&&` (0/1 results, short-circuit).

use super::bit::parse_bitor;
use super::primary::skip_ws;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn parse_or(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_and(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if !eat_op2(chars, '|', '|') {
            return Ok(left);
        }
        let right = parse_and(chars, env, last_status)?;
        left = i64::from(left != 0 || right != 0);
    }
}

fn parse_and(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_bitor(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if !eat_op2(chars, '&', '&') {
            return Ok(left);
        }
        let right = parse_bitor(chars, env, last_status)?;
        left = i64::from(left != 0 && right != 0);
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
