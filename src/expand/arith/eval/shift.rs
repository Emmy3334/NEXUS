//! Shift operators `<<` `>>`.

use super::add::parse_add;
use super::primary::skip_ws;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn parse_shift(
    chars: &mut Peekable<Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_add(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        if eat_op2(chars, '<', '<') {
            left = left.wrapping_shl(shift_amount(parse_add(chars, env, last_status)?));
        } else if eat_op2(chars, '>', '>') {
            left = left.wrapping_shr(shift_amount(parse_add(chars, env, last_status)?));
        } else {
            return Ok(left);
        }
    }
}

fn shift_amount(n: i64) -> u32 {
    u32::try_from(n.clamp(0, 63)).unwrap_or(0)
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
