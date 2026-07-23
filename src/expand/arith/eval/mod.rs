//! Integer arithmetic evaluator (`$((…))` body).

mod add;
mod assign;
mod bit;
mod compare;
mod logic;
mod primary;
mod shift;
mod ternary;

use crate::env::ShellEnvironment;
use crate::lex::LexError;
use assign::parse_assign;
use primary::skip_ws;

use std::iter::Peekable;
use std::str::Chars;

pub(super) fn evaluate(
    body: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    let mut chars = body.chars().peekable();
    let value = parse_assign(&mut chars, env, last_status)?;
    skip_ws(&mut chars);
    if chars.peek().is_some() {
        return Err(LexError::Arithmetic);
    }
    Ok(value)
}

/// Shared entry used by parenthesized / nested forms.
pub(super) fn parse_expr(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    parse_assign(chars, env, last_status)
}
