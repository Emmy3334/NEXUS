//! Resolve which history event a designator refers to.

use super::super::History;
use super::cursor::{bump, peek};
use super::scan::{is_prefix_continue, is_prefix_start, read_number};
use super::HistoryError;

/// Text of the selected event.
pub(super) struct EventText {
    pub(super) text: String,
}

/// Parse event after `!` has been skipped; `current` is `!#` text.
pub(super) fn take_event(
    s: &str,
    i: &mut usize,
    history: &mut History,
    current: &str,
) -> Result<EventText, HistoryError> {
    let Some(ch) = peek(s, *i) else {
        return Err(HistoryError::EventNotFound);
    };
    match ch {
        '!' => {
            bump(s, i);
            event_relative(history, 1)
        }
        '#' => {
            bump(s, i);
            Ok(EventText {
                text: current.to_string(),
            })
        }
        '?' => take_search(s, i, history),
        '{' => take_braced(s, i, history),
        '-' => take_relative(s, i, history),
        c if c.is_ascii_digit() => take_absolute_or_prefix(s, i, history),
        c if is_prefix_start(c) => take_prefix(s, i, history),
        ':' | '^' | '$' | '*' | '%' => event_relative(history, 1),
        _ => Err(HistoryError::EventNotFound),
    }
}

fn event_relative(history: &History, n: usize) -> Result<EventText, HistoryError> {
    match history.get_relative(n) {
        Some(text) => Ok(EventText {
            text: text.to_string(),
        }),
        None => Err(HistoryError::EventNotFound),
    }
}

fn take_relative(s: &str, i: &mut usize, history: &History) -> Result<EventText, HistoryError> {
    bump(s, i);
    let Some(n) = read_number(s, i) else {
        return Err(HistoryError::EventNotFound);
    };
    event_relative(history, n)
}

fn take_absolute_or_prefix(
    s: &str,
    i: &mut usize,
    history: &History,
) -> Result<EventText, HistoryError> {
    let start = *i;
    let Some(n) = read_number(s, i) else {
        return Err(HistoryError::EventNotFound);
    };
    if let Some(c) = peek(s, *i) {
        if is_prefix_continue(c) && !c.is_ascii_digit() {
            *i = start;
            return take_prefix(s, i, history);
        }
    }
    match history.get(n) {
        Some(text) => Ok(EventText {
            text: text.to_string(),
        }),
        None => Err(HistoryError::EventNotFound),
    }
}

fn take_prefix(s: &str, i: &mut usize, history: &History) -> Result<EventText, HistoryError> {
    let start = *i;
    while peek(s, *i).is_some_and(is_prefix_continue) {
        bump(s, i);
    }
    let prefix = &s[start..*i];
    match history.find_prefix(prefix) {
        Some(text) => Ok(EventText {
            text: text.to_string(),
        }),
        None => Err(HistoryError::EventNotFound),
    }
}

fn take_braced(s: &str, i: &mut usize, history: &History) -> Result<EventText, HistoryError> {
    bump(s, i);
    let start = *i;
    while peek(s, *i).is_some_and(|c| c != '}') {
        bump(s, i);
    }
    if peek(s, *i).is_none() {
        return Err(HistoryError::EventNotFound);
    }
    let prefix = &s[start..*i];
    bump(s, i);
    match history.find_prefix(prefix) {
        Some(text) => Ok(EventText {
            text: text.to_string(),
        }),
        None => Err(HistoryError::EventNotFound),
    }
}

fn take_search(s: &str, i: &mut usize, history: &mut History) -> Result<EventText, HistoryError> {
    bump(s, i);
    let start = *i;
    while peek(s, *i).is_some_and(|c| c != '?') {
        bump(s, i);
    }
    let needle = s[start..*i].to_string();
    if peek(s, *i) == Some('?') {
        bump(s, i);
    }
    let text = history
        .find_substring(&needle)
        .ok_or(HistoryError::EventNotFound)?
        .to_string();
    let word = text
        .split_whitespace()
        .find(|w| w.contains(&needle))
        .unwrap_or(&text)
        .to_string();
    history.last_search_word = Some(word);
    Ok(EventText { text })
}
