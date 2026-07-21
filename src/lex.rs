//! Lexical analysis for shell command lines (Dragon Book Ch. 3).
//!
//! Recognizes [`TokenKind::Word`] and Minishell2 operators: `;`, `|`, `>`,
//! `<`, `>>`, `<<`. Quotes and escapes arrive in later slices.

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
/// Call [`Token::lexeme`] with the same `source` that was tokenized to read
/// the text. Spans are invalid after that source buffer is mutated.
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

    /// Borrow the lexeme from `source`.
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

/// Tokenize `source` into `tokens`, reusing `tokens`' capacity.
///
/// Clears `tokens` first. One forward scan; no intermediate collections.
/// Two-character operators (`>>`, `<<`) are preferred over single `>` / `<`.
/// Operators split words even without surrounding whitespace (`ls|wc`).
pub fn tokenize_into(source: &str, tokens: &mut Vec<Token>) {
    tokens.clear();

    let mut position = 0;
    let source_len = source.len();

    while position < source_len {
        let Some(token_start) = next_non_whitespace(source, position) else {
            break;
        };
        position = push_token(source, token_start, tokens);
    }
}

fn push_token(source: &str, start: usize, tokens: &mut Vec<Token>) -> usize {
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
            let end = word_end(source, start);
            (TokenKind::Word, end)
        }
    };

    tokens.push(Token { kind, start, end });
    end
}

fn next_non_whitespace(source: &str, from: usize) -> Option<usize> {
    source[from..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(offset, _)| from + offset)
}

fn word_end(source: &str, from: usize) -> usize {
    source[from..]
        .char_indices()
        .find(|(_, ch)| ch.is_whitespace() || is_operator_char(*ch))
        .map(|(offset, _)| from + offset)
        .unwrap_or(source.len())
}

fn is_operator_char(ch: char) -> bool {
    matches!(ch, ';' | '|' | '>' | '<')
}
