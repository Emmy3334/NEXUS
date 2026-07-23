//! Quote-state transitions while scanning word boundaries.

use super::QuoteState;

/// Advance quote state for `ch` at `source[i]` (backslash handled by caller).
pub(super) fn advance_at(state: QuoteState, source: &str, i: usize) -> QuoteState {
    let ch = source[i..].chars().next().expect("i in range");
    match state {
        QuoteState::Normal => match ch {
            '\'' => QuoteState::Single,
            '"' => QuoteState::Double,
            '`' => QuoteState::Backtick,
            _ => QuoteState::Normal,
        },
        QuoteState::Single => {
            if ch == '\'' {
                QuoteState::Normal
            } else {
                QuoteState::Single
            }
        }
        QuoteState::Double => {
            if ch == '"' {
                QuoteState::Normal
            } else {
                QuoteState::Double
            }
        }
        QuoteState::Backtick => {
            if ch == '`' {
                QuoteState::Normal
            } else {
                QuoteState::Backtick
            }
        }
    }
}

/// True when `\` escapes the next character in this quote state.
pub(super) const fn escapes(state: QuoteState) -> bool {
    matches!(
        state,
        QuoteState::Normal | QuoteState::Double | QuoteState::Backtick
    )
}
