//! Parse and apply a chain of `:modifiers`.

use super::super::cursor::{bump, peek};
use super::super::scan::join_words;
use super::super::selected::Selected;
use super::super::HistoryError;
use super::path::{ext, head, lower_first, root, tail, upper_first};
use super::subst::{map_subst, quote_word, take_subst};
use crate::history::History;

/// Result of applying modifiers to a selection.
#[derive(Debug, Clone)]
pub(in super::super) struct ModifierResult {
    pub(in super::super) text: String,
    pub(in super::super) print_only: bool,
}

/// Consume zero or more `:mod` sequences and apply them.
pub(in super::super) fn apply_modifiers(
    s: &str,
    i: &mut usize,
    selected: Selected,
    history: &mut History,
) -> Result<ModifierResult, HistoryError> {
    let mut print_only = false;
    if peek(s, *i) != Some(':') {
        return Ok(ModifierResult {
            text: selected.render(),
            print_only,
        });
    }
    let mut words = selected.into_words();
    if words.is_empty() {
        return Ok(ModifierResult {
            text: String::new(),
            print_only,
        });
    }
    while peek(s, *i) == Some(':') {
        bump(s, i);
        let flags = take_flags(s, i);
        apply_one(s, i, &mut words, flags, history, &mut print_only)?;
    }
    Ok(ModifierResult {
        text: join_words(&words),
        print_only,
    })
}

#[derive(Clone, Copy, Default)]
struct Flags {
    global: bool,
    repeat: bool,
}

fn take_flags(s: &str, i: &mut usize) -> Flags {
    let mut flags = Flags::default();
    while let Some(ch) = peek(s, *i) {
        match ch {
            'g' => {
                flags.global = true;
                bump(s, i);
            }
            'a' => {
                flags.repeat = true;
                bump(s, i);
            }
            _ => break,
        }
    }
    flags
}

fn apply_one(
    s: &str,
    i: &mut usize,
    words: &mut [String],
    flags: Flags,
    history: &mut History,
    print_only: &mut bool,
) -> Result<(), HistoryError> {
    let Some(ch) = peek(s, *i) else {
        return Err(HistoryError::BadModifier);
    };
    match ch {
        'p' => {
            bump(s, i);
            *print_only = true;
            Ok(())
        }
        'h' | 't' | 'r' | 'e' | 'u' | 'l' | 'q' | 'Q' | 'x' => {
            bump(s, i);
            map_words(words, flags.global, |w| map_simple(ch, w))
        }
        's' => apply_s(s, i, words, flags, history),
        '&' => {
            bump(s, i);
            apply_amp(words, flags, history)
        }
        _ => Err(HistoryError::BadModifier),
    }
}

fn apply_s(
    s: &str,
    i: &mut usize,
    words: &mut [String],
    flags: Flags,
    history: &mut History,
) -> Result<(), HistoryError> {
    bump(s, i);
    let (left, right) = take_subst(s, i, history).ok_or(HistoryError::BadModifier)?;
    map_subst(words, flags.global, flags.repeat, &left, &right)
}

fn apply_amp(words: &mut [String], flags: Flags, history: &History) -> Result<(), HistoryError> {
    let (left, right) = history
        .last_subst
        .clone()
        .ok_or(HistoryError::BadModifier)?;
    map_subst(words, flags.global, flags.repeat, &left, &right)
}

fn map_simple(kind: char, word: &str) -> String {
    match kind {
        'h' => head(word),
        't' => tail(word),
        'r' => root(word),
        'e' => ext(word),
        'u' => upper_first(word),
        'l' => lower_first(word),
        'q' | 'Q' => quote_word(word),
        'x' => word.to_string(),
        _ => word.to_string(),
    }
}

fn map_words(
    words: &mut [String],
    global: bool,
    f: impl Fn(&str) -> String,
) -> Result<(), HistoryError> {
    if words.is_empty() {
        return Err(HistoryError::BadModifier);
    }
    if global {
        for w in words.iter_mut() {
            *w = f(w);
        }
    } else {
        words[0] = f(&words[0]);
    }
    Ok(())
}
