//! Arithmetic expansion `$((…))`.

mod eval;
mod read;

use super::ExpandedWord;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

/// Consume body after `((` already eaten; push decimal result.
pub(super) fn push_arith(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
) -> Result<(), LexError> {
    let body = read::take_body(chars)?;
    let value = eval::evaluate(&body, env, last_status)?;
    out.push_str_literal(&value.to_string());
    Ok(())
}
