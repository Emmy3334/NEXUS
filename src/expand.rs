//! Word expansion for execution: quotes/escapes, then `$` parameters.
//!
//! Quote rules match [`crate::lex::expand_word`]. Dollar expansion runs in
//! normal and double-quoted regions only (not inside `'…'`).

use crate::env::ShellEnvironment;
use crate::lex::LexError;

/// Expand a raw word for argv / redirects: strip quotes, apply escapes, expand
/// `$VAR`, `${VAR}`, `$?`, and `$status`.
pub fn expand_word_for_exec(
    raw: &str,
    env: &ShellEnvironment,
    last_status: u8,
) -> Result<String, LexError> {
    let mut out = String::with_capacity(raw.len());
    expand_word_for_exec_into(raw, env, last_status, &mut out)?;
    Ok(out)
}

/// Like [`expand_word_for_exec`], writing into `out` (cleared first).
pub fn expand_word_for_exec_into(
    raw: &str,
    env: &ShellEnvironment,
    last_status: u8,
    out: &mut String,
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
                        out.push(next);
                    }
                }
                '$' => push_parameter(&mut chars, env, last_status, out),
                _ => out.push(ch),
            },
            QuoteState::Single => {
                if ch == '\'' {
                    state = QuoteState::Normal;
                } else {
                    out.push(ch);
                }
            }
            QuoteState::Double => match ch {
                '"' => state = QuoteState::Normal,
                '\\' => match chars.next() {
                    // Escaped `$` stays literal (do not expand).
                    Some(next) if matches!(next, '"' | '\\' | '$' | '`' | '\n') => out.push(next),
                    Some(next) => {
                        out.push('\\');
                        out.push(next);
                    }
                    None => {}
                },
                '$' => push_parameter(&mut chars, env, last_status, out),
                _ => out.push(ch),
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
    out: &mut String,
) {
    match chars.peek().copied() {
        Some('?') => {
            chars.next();
            let _ = std::fmt::Write::write_fmt(out, format_args!("{last_status}"));
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
                // Unclosed `${` — treat as literal `${` + gathered text.
                out.push('$');
                out.push('{');
                out.push_str(&name);
                return;
            }
            if name.is_empty() {
                // `${}` → empty
                return;
            }
            push_named_parameter(&name, env, last_status, out);
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
            push_named_parameter(&name, env, last_status, out);
        }
        _ => out.push('$'),
    }
}

fn push_named_parameter(name: &str, env: &ShellEnvironment, last_status: u8, out: &mut String) {
    if name == "status" {
        let _ = std::fmt::Write::write_fmt(out, format_args!("{last_status}"));
        return;
    }
    if let Some(value) = env.lookup(name) {
        out.push_str(value);
    }
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
