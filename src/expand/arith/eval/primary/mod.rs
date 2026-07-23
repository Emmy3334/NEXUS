//! Unary / primary / number / bare name / nested `$((…))`.

mod dollar;

use super::parse_expr;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn skip_ws(chars: &mut Peekable<Chars<'_>>) {
    while matches!(chars.peek(), Some(c) if c.is_whitespace()) {
        chars.next();
    }
}

pub(super) fn parse_unary(
    chars: &mut Peekable<Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    skip_ws(chars);
    match chars.peek().copied() {
        Some('+') => {
            chars.next();
            parse_unary(chars, env, last_status)
        }
        Some('-') => {
            chars.next();
            Ok(parse_unary(chars, env, last_status)?.wrapping_neg())
        }
        Some('!') => {
            chars.next();
            Ok(i64::from(parse_unary(chars, env, last_status)? == 0))
        }
        Some('~') => {
            chars.next();
            Ok(!parse_unary(chars, env, last_status)?)
        }
        _ => parse_primary(chars, env, last_status),
    }
}

fn parse_primary(
    chars: &mut Peekable<Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    skip_ws(chars);
    match chars.peek().copied() {
        Some('(') => {
            chars.next();
            let value = parse_expr(chars, env, last_status)?;
            skip_ws(chars);
            if chars.next() != Some(')') {
                return Err(LexError::Arithmetic);
            }
            Ok(value)
        }
        Some('$') => {
            chars.next();
            dollar::value(chars, env, last_status)
        }
        Some(c) if c.is_ascii_digit() => parse_number(chars),
        Some(c) if dollar::is_name_start(c) => Ok(dollar::lookup_int(
            &dollar::take_name(chars),
            env,
            last_status,
        )),
        _ => Err(LexError::Arithmetic),
    }
}

fn parse_number(chars: &mut Peekable<Chars<'_>>) -> Result<i64, LexError> {
    let mut text = String::new();
    while let Some(c) = chars.peek().copied() {
        if !c.is_ascii_digit() {
            break;
        }
        text.push(c);
        chars.next();
    }
    text.parse().map_err(|_| LexError::Arithmetic)
}
