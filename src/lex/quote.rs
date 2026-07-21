//! Quote-state transitions while scanning word boundaries.

use super::QuoteState;

/// Advance the quote state by one character, consuming an escaped character
/// from `chars` when `ch` is a backslash in an escapable position.
pub(super) fn advance(
    state: QuoteState,
    ch: char,
    chars: &mut std::str::CharIndices<'_>,
) -> QuoteState {
    match state {
        QuoteState::Normal => advance_normal(ch, chars),
        QuoteState::Single => {
            if ch == '\'' {
                QuoteState::Normal
            } else {
                QuoteState::Single
            }
        }
        QuoteState::Double => advance_pair(ch, chars, '"', QuoteState::Double),
        QuoteState::Backtick => advance_pair(ch, chars, '`', QuoteState::Backtick),
    }
}

fn advance_normal(ch: char, chars: &mut std::str::CharIndices<'_>) -> QuoteState {
    match ch {
        '\'' => QuoteState::Single,
        '"' => QuoteState::Double,
        '`' => QuoteState::Backtick,
        '\\' => {
            let _ = chars.next();
            QuoteState::Normal
        }
        _ => QuoteState::Normal,
    }
}

fn advance_pair(
    ch: char,
    chars: &mut std::str::CharIndices<'_>,
    closer: char,
    inside: QuoteState,
) -> QuoteState {
    match ch {
        '\\' => {
            let _ = chars.next();
            inside
        }
        c if c == closer => QuoteState::Normal,
        _ => inside,
    }
}
