//! Splitting a source line into [`Token`] spans.

use super::{LexError, QuoteState, Token, TokenKind};

/// Tokenize `source` into `tokens`, reusing `tokens`' capacity.
///
/// Clears `tokens` first. One forward scan; no intermediate collections.
/// Two-character operators (`>>`, `<<`) are preferred over single `>` / `<`.
/// Operators split words only when outside quotes. `\` escapes the next
/// character outside quotes (and a few insides `"…"`).
pub fn tokenize_into(source: &str, tokens: &mut Vec<Token>) -> Result<(), LexError> {
    tokens.clear();

    let mut position = 0;
    let source_len = source.len();

    while position < source_len {
        let Some(token_start) = next_non_whitespace(source, position) else {
            break;
        };
        position = push_token(source, token_start, tokens)?;
    }
    Ok(())
}

fn push_token(source: &str, start: usize, tokens: &mut Vec<Token>) -> Result<usize, LexError> {
    let bytes = source.as_bytes();
    let first = bytes[start];

    let (kind, end) = match first {
        b';' => (TokenKind::Semicolon, start + 1),
        b'|' => (TokenKind::Pipe, start + 1),
        b'>' => {
            if bytes.get(start + 1) == Some(&b'>') {
                (TokenKind::RedirectAppend, start + 2)
            } else {
                (TokenKind::RedirectOut, start + 1)
            }
        }
        b'<' => {
            if bytes.get(start + 1) == Some(&b'<') {
                (TokenKind::Heredoc, start + 2)
            } else {
                (TokenKind::RedirectIn, start + 1)
            }
        }
        _ => {
            let end = scan_word_end(source, start)?;
            (TokenKind::Word, end)
        }
    };

    tokens.push(Token { kind, start, end });
    Ok(end)
}

fn next_non_whitespace(source: &str, from: usize) -> Option<usize> {
    source[from..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, _)| from + offset)
}

/// Scan one word starting at `from` (must not be whitespace/operator).
fn scan_word_end(source: &str, from: usize) -> Result<usize, LexError> {
    let mut state = QuoteState::Normal;
    let mut chars = source[from..].char_indices();

    while let Some((rel, ch)) = chars.next() {
        if state == QuoteState::Normal && (ch.is_whitespace() || is_operator_char(ch)) {
            return Ok(from + rel);
        }
        state = advance_quote_state(state, ch, &mut chars);
    }

    if state != QuoteState::Normal {
        return Err(LexError::UnclosedQuote);
    }
    Ok(source.len())
}

/// Advance the quote state by one character, consuming an escaped character
/// from `chars` when `ch` is a backslash in an escapable position.
fn advance_quote_state(
    state: QuoteState,
    ch: char,
    chars: &mut std::str::CharIndices<'_>,
) -> QuoteState {
    match state {
        QuoteState::Normal => match ch {
            '\'' => QuoteState::Single,
            '"' => QuoteState::Double,
            '\\' => {
                let _ = chars.next();
                QuoteState::Normal
            }
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
            if ch == '\\' {
                let _ = chars.next();
                QuoteState::Double
            } else if ch == '"' {
                QuoteState::Normal
            } else {
                QuoteState::Double
            }
        }
    }
}

fn is_operator_char(ch: char) -> bool {
    matches!(ch, ';' | '|' | '>' | '<')
}
