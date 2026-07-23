//! Additive and multiplicative operators (`+` `-` `*` `/` `%` `**`).

use super::primary::{parse_unary, skip_ws};
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn parse_add(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_mul(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        match chars.peek().copied() {
            Some('+') => {
                chars.next();
                left = left.wrapping_add(parse_mul(chars, env, last_status)?);
            }
            Some('-') => {
                chars.next();
                left = left.wrapping_sub(parse_mul(chars, env, last_status)?);
            }
            _ => return Ok(left),
        }
    }
}

fn parse_mul(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_power(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        match chars.peek().copied() {
            Some('*') if !peek_second(chars, '*') => {
                chars.next();
                left = left.wrapping_mul(parse_power(chars, env, last_status)?);
            }
            Some('/') => {
                chars.next();
                let right = parse_power(chars, env, last_status)?;
                if right == 0 {
                    return Err(LexError::Arithmetic);
                }
                left /= right;
            }
            Some('%') => {
                chars.next();
                let right = parse_power(chars, env, last_status)?;
                if right == 0 {
                    return Err(LexError::Arithmetic);
                }
                left %= right;
            }
            _ => return Ok(left),
        }
    }
}

fn parse_power(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let base = parse_unary(chars, env, last_status)?;
    skip_ws(chars);
    if !eat_op2(chars, '*', '*') {
        return Ok(base);
    }
    let exp = parse_power(chars, env, last_status)?;
    ipow(base, exp)
}

fn ipow(base: i64, exp: i64) -> Result<i64, LexError> {
    if exp < 0 {
        return Err(LexError::Arithmetic);
    }
    let mut result = 1i64;
    let mut b = base;
    let mut e = exp as u64;
    while e > 0 {
        if e & 1 == 1 {
            result = result.wrapping_mul(b);
        }
        b = b.wrapping_mul(b);
        e >>= 1;
    }
    Ok(result)
}

fn peek_second(chars: &Peekable<Chars<'_>>, want: char) -> bool {
    let mut clone = chars.clone();
    clone.next();
    clone.peek() == Some(&want)
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
