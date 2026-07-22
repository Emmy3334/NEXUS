//! tcsh-style `foreach` / `end` control structure.

mod header;

use crate::lex::{Token, TokenKind};
pub use header::{parse_header, ForEachHeader};

/// True when `line` is a lone `end` keyword (body terminator).
#[must_use]
pub fn is_end_line(line: &str) -> bool {
    line.trim() == "end"
}

/// True when lexing `line` yields a `foreach` header (for nested depth).
#[must_use]
pub fn line_opens_foreach(line: &str) -> bool {
    let mut tokens = Vec::new();
    if crate::lex::tokenize_into(line, &mut tokens).is_err() {
        return false;
    }
    parse_header(line, &tokens).is_some()
}

/// First token is the `foreach` keyword (valid header or not).
#[must_use]
pub fn starts_with_foreach(source: &str, tokens: &[Token]) -> bool {
    matches!(
        tokens.first(),
        Some(t) if t.kind == TokenKind::Word && t.lexeme(source) == "foreach"
    )
}

/// stderr message for an incomplete / invalid foreach line.
#[must_use]
pub fn foreach_syntax_message(_source: &str, _tokens: &[Token]) -> &'static str {
    "foreach: Words not parenthesized."
}
