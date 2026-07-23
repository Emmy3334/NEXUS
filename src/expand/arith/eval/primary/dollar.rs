//! `$name` / `$?` / `$n` / nested `$((…))` (unset → 0).

use super::super::super::read;
use super::super::evaluate;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn value(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    match chars.peek().copied() {
        Some('?') => {
            chars.next();
            Ok(i64::from(last_status))
        }
        Some('(') => nested_arith(chars, env, last_status),
        Some(c) if c.is_ascii_digit() => Ok(lookup_int(&take_digits(chars), env, last_status)),
        Some(c) if is_name_start(c) => Ok(lookup_int(&take_name(chars), env, last_status)),
        _ => Ok(0),
    }
}

fn nested_arith(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    chars.next(); // '('
    if chars.peek() != Some(&'(') {
        return Err(LexError::Arithmetic);
    }
    chars.next(); // second '('
    let body = read::take_body(chars)?;
    evaluate(&body, env, last_status)
}

fn take_digits(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut name = String::new();
    while let Some(d) = chars.peek().copied() {
        if !d.is_ascii_digit() {
            break;
        }
        name.push(d);
        chars.next();
    }
    name
}

pub(in crate::expand::arith::eval) fn take_name(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut name = String::new();
    if let Some(c) = chars.next() {
        name.push(c);
    }
    while let Some(n) = chars.peek().copied() {
        if !is_name_continue(n) {
            break;
        }
        name.push(n);
        chars.next();
    }
    name
}

pub(in crate::expand::arith::eval) fn lookup_int(
    name: &str,
    env: &ShellEnvironment,
    last_status: u8,
) -> i64 {
    if name == "status" {
        return i64::from(last_status);
    }
    let text = if let Ok(index) = name.parse::<usize>() {
        env.positional(index).unwrap_or("")
    } else {
        env.lookup(name).unwrap_or("")
    };
    if text.is_empty() {
        return 0;
    }
    text.parse().unwrap_or(0)
}

pub(in crate::expand::arith::eval) fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
