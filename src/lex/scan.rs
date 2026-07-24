//! Splitting a source line into [`Token`] spans.

use super::{
    arith_span, brace_param_span, cmd_subst_span, quote, LexError, QuoteState, Token, TokenKind,
};

/// Tokenize `source` into `tokens`, reusing `tokens`' capacity.
///
/// Clears `tokens` first. One forward scan; no intermediate collections.
/// Two-character operators (`>>`, `<<`) are preferred over single `>` / `<`.
/// Operators split words only when outside quotes. `\` escapes the next
/// character outside quotes (and a few insides `"…"`). `$((…))` / `$(…)` /
/// bare `((…))` stay one word.
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
    let (kind, end) = match bytes[start] {
        b';' => (TokenKind::Semicolon, start + 1),
        b'(' => match arith_span::try_close_cmd_arith(source, start)? {
            Some(end) => (TokenKind::Word, end),
            None => (TokenKind::LParen, start + 1),
        },
        b')' => (TokenKind::RParen, start + 1),
        b'|' | b'&' | b'>' | b'<' => two_char_op(bytes, start),
        _ => {
            let end = scan_word_end(source, start)?;
            (TokenKind::Word, end)
        }
    };

    tokens.push(Token { kind, start, end });
    Ok(end)
}

fn two_char_op(bytes: &[u8], start: usize) -> (TokenKind, usize) {
    let next = bytes.get(start + 1).copied();
    match bytes[start] {
        b'|' if next == Some(b'|') => (TokenKind::OrOr, start + 2),
        b'|' => (TokenKind::Pipe, start + 1),
        b'&' if next == Some(b'&') => (TokenKind::AndAnd, start + 2),
        b'&' => (TokenKind::Ampersand, start + 1),
        b'>' if next == Some(b'>') => (TokenKind::RedirectAppend, start + 2),
        b'>' => (TokenKind::RedirectOut, start + 1),
        b'<' if next == Some(b'<') => (TokenKind::Heredoc, start + 2),
        _ => (TokenKind::RedirectIn, start + 1),
    }
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
    let mut i = from;
    while i < source.len() {
        let ch = source[i..].chars().next().expect("i in range");
        let ch_len = ch.len_utf8();
        if state == QuoteState::Normal {
            if ch.is_whitespace() {
                return Ok(i);
            }
            if let Some(end) = arith_span::try_close_arith(source, i)? {
                i = end;
                continue;
            }
            if let Some(end) = arith_span::try_close_cmd_arith(source, i)? {
                i = end;
                continue;
            }
            if let Some(end) = cmd_subst_span::try_close(source, i)? {
                i = end;
                continue;
            }
            if let Some(end) = brace_param_span::try_close(source, i)? {
                i = end;
                continue;
            }
            if is_operator_char(ch) {
                return Ok(i);
            }
        }
        if quote::escapes(state) && ch == '\\' {
            i += ch_len;
            if let Some(next) = source[i..].chars().next() {
                i += next.len_utf8();
            }
            continue;
        }
        state = quote::advance_at(state, source, i);
        i += ch_len;
    }
    if state != QuoteState::Normal {
        return Err(LexError::UnclosedQuote);
    }
    Ok(source.len())
}

fn is_operator_char(ch: char) -> bool {
    matches!(ch, ';' | '|' | '&' | '>' | '<' | '(' | ')')
}
