//! Lexical analysis for shell command lines (Dragon Book Ch. 3).
//!
//! Recognizes [`TokenKind::Word`] and operators: `;`, `|`, `&`, `>`, `<`, `>>`,
//! `<<`, `(`, `)`. Words may contain `'…'`, `"…"`, and `` `…` `` / `\` escapes
//! so that spaces and operators inside quotes stay part of the word.

mod expand;
mod quote;
mod scan;

pub use expand::{expand_word, expand_word_into};
pub use scan::tokenize_into;

/// What kind of lexeme a [`Token`] spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    Semicolon,
    Pipe,
    /// `&` — background the preceding pipeline.
    Ampersand,
    RedirectOut,
    RedirectAppend,
    RedirectIn,
    Heredoc,
    /// `(` — start of a subshell / grouping.
    LParen,
    /// `)` — end of a subshell / grouping.
    RParen,
}

impl TokenKind {
    /// Whether this kind is a shell operator (not a word).
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
    /// A `'`, `"`, or `` ` `` was opened and never closed on this line.
    UnclosedQuote,
    /// Command substitution (`` `…` ``) failed to run.
    CommandSubstitution,
}

impl LexError {
    /// Human-readable message for stderr.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnclosedQuote => "Unmatched quote.",
            Self::CommandSubstitution => "Command substitution failed.",
        }
    }
}

/// Shared quote-tracking state used while scanning word boundaries and while
/// expanding a word's runtime text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteState {
    Normal,
    Single,
    Double,
    /// `` `…` `` — command substitution span (keeps spaces/operators inside).
    Backtick,
}
