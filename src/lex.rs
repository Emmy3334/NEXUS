//! Lexical analysis for shell command lines (Dragon Book Ch. 3).
//!
//! Recognizes [`TokenKind::Word`] and Minishell2 operators: `;`, `|`, `>`,
//! `<`, `>>`, `<<`. Words may contain `'…'`, `"…"`, and `\` escapes so that
//! spaces and operators inside quotes stay part of the word.

/// What kind of lexeme a [`Token`] spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    Semicolon,
    Pipe,
    RedirectOut,
    RedirectAppend,
    RedirectIn,
    Heredoc,
}

impl TokenKind {
    /// Whether this kind is a Minishell2 operator (not a word).
    #[must_use]
    pub const fn is_operator(self) -> bool {
        !matches!(self, Self::Word)
    }
}

/// A token stored as a byte span into the source line (no heap copy).
///
/// Word spans may include quote and backslash characters. Use
/// [`expand_word`] / [`expand_word_into`] to get the runtime argv text.
///
/// Call [`Token::lexeme`] with the same `source` that was tokenized to read
/// the raw span. Spans are invalid after that source buffer is mutated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

impl Token {
    /// Borrow the lexeme from `source` if the span is in range.
    pub fn try_lexeme<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.get(self.start..self.end)
    }

    /// Borrow the raw lexeme from `source` (quotes/escapes not stripped).
    ///
    /// # Panics
    ///
    /// Panics if `start..end` is not a valid byte range in `source`
    /// (caller must pass the same string that produced this token).
    pub fn lexeme<'a>(&self, source: &'a str) -> &'a str {
        self.try_lexeme(source).unwrap_or_else(|| {
            panic!(
                "token span {}..{} is invalid for source of length {}",
                self.start,
                self.end,
                source.len()
            )
        })
    }
}

/// Why tokenization or word expansion failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexError {
    /// A `'` or `"` was opened and never closed on this line.
    UnclosedQuote,
}

impl LexError {
    /// Human-readable message for stderr.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnclosedQuote => "Unmatched quote.",
        }
    }
}

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

/// Expand a raw word lexeme: strip quotes and apply backslash escapes.
pub fn expand_word(raw: &str) -> Result<String, LexError> {
    let mut out = String::with_capacity(raw.len());
    expand_word_into(raw, &mut out)?;
    Ok(out)
}

/// Expand `raw` into `out`, clearing `out` first and reusing its capacity.
pub fn expand_word_into(raw: &str, out: &mut String) -> Result<(), LexError> {
    out.clear();
    let mut chars = raw.chars();
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
                    Some(next) if matches!(next, '"' | '\\' | '$' | '`' | '\n') => out.push(next),
                    Some(next) => {
                        out.push('\\');
                        out.push(next);
                    }
                    None => {}
                },
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
        let abs = from + rel;
        match state {
            QuoteState::Normal => {
                if ch.is_whitespace() || is_operator_char(ch) {
                    return Ok(abs);
                }
                match ch {
                    '\'' => state = QuoteState::Single,
                    '"' => state = QuoteState::Double,
                    '\\' => {
                        let _ = chars.next();
                    }
                    _ => {}
                }
            }
            QuoteState::Single => {
                if ch == '\'' {
                    state = QuoteState::Normal;
                }
            }
            QuoteState::Double => {
                if ch == '\\' {
                    let _ = chars.next();
                } else if ch == '"' {
                    state = QuoteState::Normal;
                }
            }
        }
    }

    if state != QuoteState::Normal {
        return Err(LexError::UnclosedQuote);
    }
    Ok(source.len())
}

fn is_operator_char(ch: char) -> bool {
    matches!(ch, ';' | '|' | '>' | '<')
}
