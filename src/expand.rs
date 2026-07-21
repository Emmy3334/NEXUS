//! Quote-aware word expansion: escapes, `$` parameters, then pathname globbing.
//!
//! Dollar expansion runs in normal and double-quoted regions (not `'…'`).
//! Glob metacharacters (`*`, `?`, `[…]`) are active only when unquoted (and
//! for unquoted `$` expansions whose values contain those characters).

use crate::env::ShellEnvironment;
use crate::lex::LexError;

/// Result of quote / `$` expansion, with per-character glob activity flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExpandedWord {
    text: String,
    /// Parallel to `text.chars()`: whether that character is an active glob meta.
    glob_meta: Vec<bool>,
}

impl ExpandedWord {
    /// Expanded text (quotes already stripped).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Consume into an owned string (drops glob flags).
    #[must_use]
    pub fn into_string(self) -> String {
        self.text
    }

    /// Whether any active glob metacharacter is present.
    #[must_use]
    pub fn has_active_glob(&self) -> bool {
        self.glob_meta.iter().any(|&m| m)
    }

    /// Borrow the parallel glob-activity flags (one per Unicode scalar in `text`).
    #[must_use]
    pub fn glob_meta(&self) -> &[bool] {
        &self.glob_meta
    }

    fn clear(&mut self) {
        self.text.clear();
        self.glob_meta.clear();
    }

    fn push_literal(&mut self, c: char) {
        self.text.push(c);
        self.glob_meta.push(false);
    }

    fn push_glob_meta(&mut self, c: char) {
        debug_assert!(matches!(c, '*' | '?' | '['));
        self.text.push(c);
        self.glob_meta.push(true);
    }

    fn push_str_literal(&mut self, s: &str) {
        for c in s.chars() {
            self.push_literal(c);
        }
    }

    /// Unquoted expansion: `*`, `?`, `[` in the value become active metas.
    fn push_str_globable(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '*' | '?' | '[' => self.push_glob_meta(c),
                _ => self.push_literal(c),
            }
        }
    }
}

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
        match state {
            QuoteState::Normal => match ch {
                '\'' => state = QuoteState::Single,
                '"' => state = QuoteState::Double,
                '\\' => {
                    if let Some(next) = chars.next() {
                        out.push_literal(next);
                    }
                }
                '*' | '?' | '[' => out.push_glob_meta(ch),
                '$' => push_parameter(&mut chars, env, last_status, out, true),
                _ => out.push_literal(ch),
            },
            QuoteState::Single => {
                if ch == '\'' {
                    state = QuoteState::Normal;
                } else {
                    out.push_literal(ch);
                }
            }
            QuoteState::Double => match ch {
                '"' => state = QuoteState::Normal,
                '\\' => match chars.next() {
                    Some(next) if matches!(next, '"' | '\\' | '$' | '`' | '\n') => {
                        out.push_literal(next);
                    }
                    Some(next) => {
                        out.push_literal('\\');
                        out.push_literal(next);
                    }
                    None => {}
                },
                '$' => push_parameter(&mut chars, env, last_status, out, false),
                _ => out.push_literal(ch),
            },
        }
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

fn push_parameter(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    match chars.peek().copied() {
        Some('?') => {
            chars.next();
            let mut buf = String::new();
            let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{last_status}"));
            out.push_str_literal(&buf);
        }
        Some('{') => {
            chars.next();
            let mut name = String::new();
            let mut closed = false;
            for ch in chars.by_ref() {
                if ch == '}' {
                    closed = true;
                    break;
                }
                name.push(ch);
            }
            if !closed {
                out.push_literal('$');
                out.push_literal('{');
                out.push_str_literal(&name);
                return;
            }
            if name.is_empty() {
                return;
            }
            push_named_parameter(&name, env, last_status, out, globable);
        }
        Some(c) if is_name_start(c) => {
            let mut name = String::new();
            name.push(c);
            chars.next();
            while let Some(c) = chars.peek().copied() {
                if !is_name_continue(c) {
                    break;
                }
                name.push(c);
                chars.next();
            }
            push_named_parameter(&name, env, last_status, out, globable);
        }
        _ => out.push_literal('$'),
    }
}

fn push_named_parameter(
    name: &str,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
    globable: bool,
) {
    if name == "status" {
        let mut buf = String::new();
        let _ = std::fmt::Write::write_fmt(&mut buf, format_args!("{last_status}"));
        out.push_str_literal(&buf);
        return;
    }
    let Some(value) = env.lookup(name) else {
        return;
    };
    if globable {
        out.push_str_globable(value);
    } else {
        out.push_str_literal(value);
    }
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
