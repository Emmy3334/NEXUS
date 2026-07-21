//! Quote-state machine that fills an [`ExpandedWord`].

use super::dollar::push_parameter;
use super::word::ExpandedWord;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

/// Expand a raw word: strip quotes, apply escapes, expand `$` / `$?` / `$status`.
pub fn expand_word_for_exec(
    raw: &str,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<ExpandedWord, LexError> {
    let mut out = ExpandedWord::default();
    expand_word_for_exec_into(raw, env, last_status, &mut out)?;
    Ok(out)
}

/// Like [`expand_word_for_exec`], writing into `out` (cleared first).
pub fn expand_word_for_exec_into(
    raw: &str,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
) -> Result<(), LexError> {
    out.clear();
    let mut chars = raw.chars().peekable();
    let mut state = QuoteState::Normal;
    while let Some(ch) = chars.next() {
        state = step(ch, state, &mut chars, env, last_status, out);
    }
    if state != QuoteState::Normal {
        return Err(LexError::UnclosedQuote);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteState {
    Normal,
    Single,
    Double,
}

fn step(
    ch: char,
    state: QuoteState,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
) -> QuoteState {
    match state {
        QuoteState::Normal => step_normal(ch, chars, env, last_status, out),
        QuoteState::Single => {
            if ch == '\'' {
                QuoteState::Normal
            } else {
                out.push_literal(ch);
                QuoteState::Single
            }
        }
        QuoteState::Double => step_double(ch, chars, env, last_status, out),
    }
}

fn step_normal(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
) -> QuoteState {
    match ch {
        '\'' => QuoteState::Single,
        '"' => QuoteState::Double,
        '\\' => {
            if let Some(next) = chars.next() {
                out.push_literal(next);
            }
            QuoteState::Normal
        }
        '*' | '?' | '[' => {
            out.push_glob_meta(ch);
            QuoteState::Normal
        }
        '$' => {
            push_parameter(chars, env, last_status, out, true);
            QuoteState::Normal
        }
        _ => {
            out.push_literal(ch);
            QuoteState::Normal
        }
    }
}

fn step_double(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
) -> QuoteState {
    match ch {
        '"' => QuoteState::Normal,
        '\\' => {
            match chars.next() {
                Some(next) if matches!(next, '"' | '\\' | '$' | '`' | '\n') => {
                    out.push_literal(next);
                }
                Some(next) => {
                    out.push_literal('\\');
                    out.push_literal(next);
                }
                None => {}
            }
            QuoteState::Double
        }
        '$' => {
            push_parameter(chars, env, last_status, out, false);
            QuoteState::Double
        }
        _ => {
            out.push_literal(ch);
            QuoteState::Double
        }
    }
}
