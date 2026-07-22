//! tcsh-style `if (expr) then` / `else` / `endif` control structure.

mod header;

pub use header::{parse_else_if, parse_header, IfHeader};

use crate::lex::{Token, TokenKind};

/// True when lexing `line` yields `if ( … ) then`.
#[must_use]
pub fn line_opens_if(line: &str) -> bool {
    let mut tokens = Vec::new();
    if crate::lex::tokenize_into(line, &mut tokens).is_err() {
        return false;
    }
    parse_header(line, &tokens).is_some()
}

/// First token is `if` (valid header or not).
#[must_use]
pub fn starts_with_if(source: &str, tokens: &[Token]) -> bool {
    matches!(
        tokens.first(),
        Some(t) if t.kind == TokenKind::Word && t.lexeme(source) == "if"
    )
}

/// Lone `else` keyword.
#[must_use]
pub fn is_else_line(line: &str) -> bool {
    line.trim() == "else"
}

/// Lone `endif` keyword.
#[must_use]
pub fn is_endif_line(line: &str) -> bool {
    line.trim() == "endif"
}

/// stderr message for an incomplete / invalid if line.
#[must_use]
pub fn if_syntax_message(_source: &str, _tokens: &[Token]) -> &'static str {
    "if: Expression Syntax."
}
