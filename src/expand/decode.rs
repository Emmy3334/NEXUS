//! Quote-state machine that fills expanded field(s) (no brace expand).

use super::backtick::push_backtick;
use super::cmd_subst::{self, push_dollar_paren};
use super::dollar::push_parameter;
use super::fields::FieldBuilder;
use super::word::ExpandedWord;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

/// Expand one brace-expanded piece into field(s).
pub(super) fn expand_word_fields_into(
    raw: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    fields_out: &mut Vec<ExpandedWord>,
    capture: &mut dyn FnMut(&str) -> Result<String, LexError>,
) -> Result<(), LexError> {
    let mut builder = FieldBuilder::new();
    let mut chars = raw.chars().peekable();
    let mut state = QuoteState::Normal;
    while let Some(ch) = chars.next() {
        state = step(
            ch,
            state,
            &mut chars,
            env,
            last_status,
            &mut builder,
            capture,
        )?;
    }
    if state != QuoteState::Normal {
        return Err(LexError::UnclosedQuote);
    }
    *fields_out = builder.into_fields();
    Ok(())
}

/// Expand without brace (parameter operator words / patterns).
pub(super) fn expand_word_for_exec(
    raw: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<ExpandedWord, LexError> {
    let mut fields = Vec::new();
    let mut deny = |_: &str| Err(LexError::CommandSubstitution);
    expand_word_fields_into(raw, env, last_status, &mut fields, &mut deny)?;
    Ok(fields.into_iter().next().unwrap_or_default())
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
    env: &mut ShellEnvironment,
    last_status: u8,
    fields: &mut FieldBuilder,
    capture: &mut dyn FnMut(&str) -> Result<String, LexError>,
) -> Result<QuoteState, LexError> {
    match state {
        QuoteState::Normal => step_normal(ch, chars, env, last_status, fields, capture),
        QuoteState::Single => {
            if ch == '\'' {
                Ok(QuoteState::Normal)
            } else {
                fields.current().push_literal(ch);
                Ok(QuoteState::Single)
            }
        }
        QuoteState::Double => step_double(ch, chars, env, last_status, fields, capture),
    }
}

fn step_normal(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
    fields: &mut FieldBuilder,
    capture: &mut dyn FnMut(&str) -> Result<String, LexError>,
) -> Result<QuoteState, LexError> {
    match ch {
        '\'' => Ok(QuoteState::Single),
        '"' => Ok(QuoteState::Double),
        '`' => {
            push_backtick(chars, fields, true, capture)?;
            Ok(QuoteState::Normal)
        }
        '\\' => {
            if let Some(next) = chars.next() {
                fields.current().push_literal(next);
            }
            Ok(QuoteState::Normal)
        }
        '*' | '?' | '[' => {
            fields.current().push_glob_meta(ch);
            Ok(QuoteState::Normal)
        }
        '$' => {
            if cmd_subst::looks_like(chars) {
                push_dollar_paren(chars, fields, true, capture)?;
            } else {
                push_parameter(chars, env, last_status, fields.current(), true)?;
            }
            Ok(QuoteState::Normal)
        }
        _ => {
            fields.current().push_literal(ch);
            Ok(QuoteState::Normal)
        }
    }
}

fn step_double(
    ch: char,
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
    fields: &mut FieldBuilder,
    capture: &mut dyn FnMut(&str) -> Result<String, LexError>,
) -> Result<QuoteState, LexError> {
    match ch {
        '"' => Ok(QuoteState::Normal),
        '`' => {
            push_backtick(chars, fields, false, capture)?;
            Ok(QuoteState::Double)
        }
        '\\' => {
            escape_double(chars, fields);
            Ok(QuoteState::Double)
        }
        '$' => {
            if cmd_subst::looks_like(chars) {
                push_dollar_paren(chars, fields, false, capture)?;
            } else {
                push_parameter(chars, env, last_status, fields.current(), false)?;
            }
            Ok(QuoteState::Double)
        }
        _ => {
            fields.current().push_literal(ch);
            Ok(QuoteState::Double)
        }
    }
}

fn escape_double(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, fields: &mut FieldBuilder) {
    match chars.next() {
        Some(next) if matches!(next, '"' | '\\' | '$' | '`' | '\n') => {
            fields.current().push_literal(next);
        }
        Some(next) => {
            fields.current().push_literal('\\');
            fields.current().push_literal(next);
        }
        None => {}
    }
}
