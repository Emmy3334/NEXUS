//! Quote-aware probe: does this word need command-substitution capture?

use std::iter::Peekable;
use std::str::Chars;

/// Whether `raw` needs `` `…` `` / `$(…)` capture (ignores quotes / `$((…))`).
#[must_use]
pub(crate) fn word_may_need_cmd_subst(raw: &str) -> bool {
    let mut chars = raw.chars().peekable();
    let mut state = QuoteState::Normal;
    while let Some(ch) = chars.next() {
        match state {
            QuoteState::Normal => match step_open(ch, &mut chars, QuoteState::Normal) {
                Probe::Found => return true,
                Probe::Continue(next) => state = next,
            },
            QuoteState::Single => {
                if ch == '\'' {
                    state = QuoteState::Normal;
                }
            }
            QuoteState::Double => match step_open(ch, &mut chars, QuoteState::Double) {
                Probe::Found => return true,
                Probe::Continue(next) => state = next,
            },
        }
    }
    false
}

#[derive(Clone, Copy)]
enum QuoteState {
    Normal,
    Single,
    Double,
}

enum Probe {
    Found,
    Continue(QuoteState),
}

fn step_open(ch: char, chars: &mut Peekable<Chars<'_>>, in_state: QuoteState) -> Probe {
    let close_quote = matches!((in_state, ch), (QuoteState::Double, '"'));
    if close_quote {
        return Probe::Continue(QuoteState::Normal);
    }
    if matches!((in_state, ch), (QuoteState::Normal, '\'')) {
        return Probe::Continue(QuoteState::Single);
    }
    if matches!((in_state, ch), (QuoteState::Normal, '"')) {
        return Probe::Continue(QuoteState::Double);
    }
    match ch {
        '\\' => {
            chars.next();
            Probe::Continue(in_state)
        }
        '`' => Probe::Found,
        '$' if is_cmd_paren(chars) => Probe::Found,
        _ => Probe::Continue(in_state),
    }
}

/// `$(…)` but not arithmetic `$((…))`.
fn is_cmd_paren(chars: &mut Peekable<Chars<'_>>) -> bool {
    let mut ahead = chars.clone();
    match ahead.next() {
        Some('(') => ahead.peek() != Some(&'('),
        _ => false,
    }
}
