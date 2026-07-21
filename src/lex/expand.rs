//! Turning a raw word lexeme into its runtime (quote/escape-stripped) text.

use super::{LexError, QuoteState};

/// Expand a raw word lexeme: strip quotes and apply backslash escapes.
pub fn expand_word(raw: &str) -> Result<String, LexError> {
    let mut out = String::with_capacity(raw.len());
    expand_word_into(raw, &mut out)?;
    Ok(out)
}

/// Expand `raw` into `out`, clearing `out` first and reusing its capacity.
pub fn expand_word_into(raw: &str, out: &mut String) -> Result<(), LexError> {
    out.clear();
    let mut chars = raw.chars();
    let mut state = QuoteState::Normal;

    while let Some(ch) = chars.next() {
        state = match state {
            QuoteState::Normal => expand_normal_char(ch, &mut chars, out),
            QuoteState::Single => expand_single_char(ch, out),
            QuoteState::Double => expand_double_char(ch, &mut chars, out),
            // Lex-only strip keeps inner text; runtime `` ` `` runs in `crate::expand`.
            QuoteState::Backtick => expand_backtick_char(ch, &mut chars, out),
        };
    }

    if state != QuoteState::Normal {
        return Err(LexError::UnclosedQuote);
    }
    Ok(())
}

fn expand_normal_char(ch: char, chars: &mut std::str::Chars<'_>, out: &mut String) -> QuoteState {
    match ch {
        '\'' => QuoteState::Single,
        '"' => QuoteState::Double,
        '`' => QuoteState::Backtick,
        '\\' => {
            if let Some(next) = chars.next() {
                out.push(next);
            }
            QuoteState::Normal
        }
        _ => {
            out.push(ch);
            QuoteState::Normal
        }
    }
}

fn expand_backtick_char(ch: char, chars: &mut std::str::Chars<'_>, out: &mut String) -> QuoteState {
    match ch {
        '`' => QuoteState::Normal,
        '\\' => {
            if let Some(next) = chars.next() {
                out.push(next);
            }
            QuoteState::Backtick
        }
        _ => {
            out.push(ch);
            QuoteState::Backtick
        }
    }
}

fn expand_single_char(ch: char, out: &mut String) -> QuoteState {
    if ch == '\'' {
        QuoteState::Normal
    } else {
        out.push(ch);
        QuoteState::Single
    }
}

fn expand_double_char(ch: char, chars: &mut std::str::Chars<'_>, out: &mut String) -> QuoteState {
    match ch {
        '"' => QuoteState::Normal,
        '\\' => {
            match chars.next() {
                Some(next) if matches!(next, '"' | '\\' | '$' | '`' | '\n') => out.push(next),
                Some(next) => {
                    out.push('\\');
                    out.push(next);
                }
                None => {}
            }
            QuoteState::Double
        }
        _ => {
            out.push(ch);
            QuoteState::Double
        }
    }
}
