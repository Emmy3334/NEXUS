//! Quote-aware `!` event expansion on a command line.

mod cursor;
mod designator;
mod event;
mod modifier;
mod outcome;
mod scan;
mod selected;
mod words;

use super::History;
use cursor::{bump, peek};
use designator::expand_designator;

pub use outcome::{ExpandOutcome, HistoryError};

/// Expand history designators outside quotes into `out`.
pub fn expand_line(
    raw: &str,
    history: &mut History,
    out: &mut String,
) -> Result<ExpandOutcome, HistoryError> {
    out.clear();
    if let Some(outcome) = try_quick_subst(raw, history, out)? {
        return Ok(outcome);
    }
    let mut i = 0;
    let mut changed = false;
    let mut print_only = false;
    let mut state = QuoteState::Normal;
    while i < raw.len() {
        state = step(
            raw,
            &mut i,
            state,
            history,
            out,
            &mut changed,
            &mut print_only,
        )?;
    }
    Ok(ExpandOutcome {
        changed,
        print_only,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuoteState {
    Normal,
    Single,
    Double,
}

fn step(
    s: &str,
    i: &mut usize,
    state: QuoteState,
    history: &mut History,
    out: &mut String,
    changed: &mut bool,
    print_only: &mut bool,
) -> Result<QuoteState, HistoryError> {
    match state {
        QuoteState::Normal => step_normal(s, i, history, out, changed, print_only),
        QuoteState::Single => {
            let ch = bump(s, i).expect("quoted at end");
            out.push(ch);
            Ok(if ch == '\'' {
                QuoteState::Normal
            } else {
                QuoteState::Single
            })
        }
        QuoteState::Double => {
            let ch = bump(s, i).expect("quoted at end");
            out.push(ch);
            Ok(if ch == '"' {
                QuoteState::Normal
            } else {
                QuoteState::Double
            })
        }
    }
}

fn step_normal(
    s: &str,
    i: &mut usize,
    history: &mut History,
    out: &mut String,
    changed: &mut bool,
    print_only: &mut bool,
) -> Result<QuoteState, HistoryError> {
    match peek(s, *i).expect("step_normal at end") {
        '\'' => {
            bump(s, i);
            out.push('\'');
            Ok(QuoteState::Single)
        }
        '"' => {
            bump(s, i);
            out.push('"');
            Ok(QuoteState::Double)
        }
        '\\' => {
            take_escape(s, i, out);
            Ok(QuoteState::Normal)
        }
        '!' => take_bang(s, i, history, out, changed, print_only),
        ch => {
            bump(s, i);
            out.push(ch);
            Ok(QuoteState::Normal)
        }
    }
}

fn take_bang(
    s: &str,
    i: &mut usize,
    history: &mut History,
    out: &mut String,
    changed: &mut bool,
    print_only: &mut bool,
) -> Result<QuoteState, HistoryError> {
    // `!#` needs the written prefix; move it (no clone), expand, then rebuild.
    let after_bang = *i + '!'.len_utf8();
    let needs_current = peek(s, after_bang) == Some('#');
    let prefix = if needs_current {
        std::mem::take(out)
    } else {
        String::new()
    };
    let result = match expand_designator(s, i, history, &prefix) {
        Ok(result) => result,
        Err(err) => {
            if needs_current {
                *out = prefix;
            }
            return Err(err);
        }
    };
    if result.expanded {
        *changed = true;
    }
    if result.print_only {
        *print_only = true;
    }
    if needs_current {
        *out = prefix;
    }
    out.push_str(&result.text);
    Ok(QuoteState::Normal)
}

fn take_escape(s: &str, i: &mut usize, out: &mut String) {
    bump(s, i); // '\\'
    out.push('\\');
    if let Some(ch) = bump(s, i) {
        out.push(ch);
    }
}

/// `^old^new` at line start → `!:s^old^new`.
fn try_quick_subst(
    raw: &str,
    history: &mut History,
    out: &mut String,
) -> Result<Option<ExpandOutcome>, HistoryError> {
    if !raw.starts_with('^') {
        return Ok(None);
    }
    let synthetic = format!("!:s{raw}");
    let mut i = 0;
    let result = expand_designator(&synthetic, &mut i, history, "")?;
    out.push_str(&result.text);
    Ok(Some(ExpandOutcome {
        changed: true,
        print_only: result.print_only,
    }))
}
