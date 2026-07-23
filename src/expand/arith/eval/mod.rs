//! Integer arithmetic: `+ - * / %`, unary `+/-`, `(…)`, `$name` / `$?` / `$n`.

mod primary;

use crate::env::ShellEnvironment;
use crate::lex::LexError;
use primary::{parse_unary, skip_ws};

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn evaluate(
    body: &str,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut chars = body.chars().peekable();
    let value = parse_expr(&mut chars, env, last_status)?;
    skip_ws(&mut chars);
    if chars.peek().is_some() {
        return Err(LexError::Arithmetic);
    }
    Ok(value)
}

pub(super) fn parse_expr(
    chars: &mut Peekable<Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_term(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        match chars.peek().copied() {
            Some('+') => {
                chars.next();
                left = left.wrapping_add(parse_term(chars, env, last_status)?);
            }
            Some('-') => {
                chars.next();
                left = left.wrapping_sub(parse_term(chars, env, last_status)?);
            }
            _ => return Ok(left),
        }
    }
}

fn parse_term(
    chars: &mut Peekable<Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut left = parse_unary(chars, env, last_status)?;
    loop {
        skip_ws(chars);
        match chars.peek().copied() {
            Some('*') => {
                chars.next();
                left = left.wrapping_mul(parse_unary(chars, env, last_status)?);
            }
            Some('/') => {
                chars.next();
                let right = parse_unary(chars, env, last_status)?;
                if right == 0 {
                    return Err(LexError::Arithmetic);
                }
                left /= right;
            }
            Some('%') => {
                chars.next();
                let right = parse_unary(chars, env, last_status)?;
                if right == 0 {
                    return Err(LexError::Arithmetic);
                }
                left %= right;
            }
            _ => return Ok(left),
        }
    }
}
