//! Select words from an event (`:0`, `:$`, `:*`, ranges, …).

use super::cursor::{bump, peek};
use super::scan::{read_number, split_words};
use super::selected::Selected;
use super::HistoryError;
use crate::history::History;

/// Optional word designator after the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WordSel {
    All,
    Range { start: usize, end: usize },
    Args,
    Match,
}

/// Parse a word designator at `s[*i]` if present.
pub(super) fn take_word_sel(s: &str, i: &mut usize) -> Result<WordSel, HistoryError> {
    let Some(ch) = peek(s, *i) else {
        return Ok(WordSel::All);
    };
    if ch == ':' {
        if peek(s, *i + ch.len_utf8()).is_some_and(is_modifier_start) {
            return Ok(WordSel::All);
        }
        bump(s, i);
        if peek(s, *i).is_none() {
            return Err(HistoryError::BadModifier);
        }
        return parse_selector(s, i);
    }
    if matches!(ch, '^' | '$' | '*' | '%') {
        return parse_selector(s, i);
    }
    Ok(WordSel::All)
}

fn is_modifier_start(c: char) -> bool {
    matches!(
        c,
        'h' | 't' | 'r' | 'e' | 'u' | 'l' | 's' | 'p' | 'q' | 'Q' | 'x' | 'g' | 'a' | '&'
    )
}

fn parse_selector(s: &str, i: &mut usize) -> Result<WordSel, HistoryError> {
    match peek(s, *i).ok_or(HistoryError::BadModifier)? {
        '^' => {
            bump(s, i);
            Ok(WordSel::Range { start: 1, end: 1 })
        }
        '$' => {
            bump(s, i);
            Ok(WordSel::Range {
                start: usize::MAX,
                end: usize::MAX,
            })
        }
        '*' => {
            bump(s, i);
            Ok(WordSel::Args)
        }
        '%' => {
            bump(s, i);
            Ok(WordSel::Match)
        }
        '-' => parse_dash_range(s, i),
        c if c.is_ascii_digit() => parse_numbered(s, i),
        _ => Err(HistoryError::BadModifier),
    }
}

fn parse_dash_range(s: &str, i: &mut usize) -> Result<WordSel, HistoryError> {
    bump(s, i);
    if peek(s, *i).is_some_and(|c| c.is_ascii_digit()) {
        let end = read_number(s, i).unwrap_or(0);
        return Ok(WordSel::Range { start: 0, end });
    }
    Ok(WordSel::Range {
        start: 0,
        end: usize::MAX - 1,
    })
}

fn parse_numbered(s: &str, i: &mut usize) -> Result<WordSel, HistoryError> {
    let start = read_number(s, i).unwrap_or(0);
    let Some(ch) = peek(s, *i) else {
        return Ok(WordSel::Range { start, end: start });
    };
    match ch {
        '-' => {
            bump(s, i);
            if peek(s, *i).is_some_and(|c| c.is_ascii_digit()) {
                let end = read_number(s, i).unwrap_or(start);
                Ok(WordSel::Range { start, end })
            } else if peek(s, *i) == Some('*') {
                bump(s, i);
                Ok(WordSel::Range {
                    start,
                    end: usize::MAX,
                })
            } else {
                Ok(WordSel::Range {
                    start,
                    end: usize::MAX - 1,
                })
            }
        }
        '*' => {
            bump(s, i);
            Ok(WordSel::Range {
                start,
                end: usize::MAX,
            })
        }
        _ => Ok(WordSel::Range { start, end: start }),
    }
}

/// Apply selection to event text.
pub(super) fn select_words(
    event: &str,
    sel: WordSel,
    history: &History,
) -> Result<Selected, HistoryError> {
    match sel {
        WordSel::All => Ok(Selected::Verbatim(event.to_string())),
        WordSel::Match => {
            let word = history
                .last_search_word
                .as_deref()
                .ok_or(HistoryError::EventNotFound)?;
            Ok(Selected::Words(vec![word.to_string()]))
        }
        WordSel::Args => {
            let words = split_words(event);
            if words.len() <= 1 {
                Ok(Selected::Words(Vec::new()))
            } else {
                Ok(Selected::Words(words[1..].to_vec()))
            }
        }
        WordSel::Range { start, end } => select_range(event, start, end),
    }
}

fn select_range(event: &str, start: usize, end: usize) -> Result<Selected, HistoryError> {
    let words = split_words(event);
    if words.is_empty() {
        return Err(HistoryError::EventNotFound);
    }
    let last = words.len() - 1;
    let s = if start == usize::MAX {
        last
    } else if start == usize::MAX - 1 {
        last.saturating_sub(1)
    } else {
        start
    };
    let e = if end == usize::MAX {
        last
    } else if end == usize::MAX - 1 {
        last.saturating_sub(1)
    } else {
        end
    };
    if s > e || s >= words.len() {
        return Err(HistoryError::BadModifier);
    }
    let e = e.min(last);
    Ok(Selected::Words(words[s..=e].to_vec()))
}
