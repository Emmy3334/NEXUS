//! tcsh-style `while (expr)` / `end` control structure.

mod expr;
mod header;

pub use expr::eval_condition;
pub use header::{parse_header, WhileHeader};

use crate::lex::{Token, TokenKind};

/// True when lexing `line` yields a `while` header (for nested depth).
#[must_use]
pub fn line_opens_while(line: &str) -> bool {
    let mut tokens = Vec::new();
    if crate::lex::tokenize_into(line, &mut tokens).is_err() {
        return false;
    }
    parse_header(line, &tokens).is_some()
}

/// First token is the `while` keyword (valid header or not).
#[must_use]
pub fn starts_with_while(source: &str, tokens: &[Token]) -> bool {
    matches!(
        tokens.first(),
        Some(t) if t.kind == TokenKind::Word && t.lexeme(source) == "while"
    )
}

/// stderr message for an incomplete / invalid while line.
#[must_use]
pub fn while_syntax_message(_source: &str, _tokens: &[Token]) -> &'static str {
    "while: Expression Syntax."
}
