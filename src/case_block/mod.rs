//! bash-style `case` / `esac` control structure.

mod header;
mod pattern;

pub use header::{parse_header, CaseHeader};
pub use pattern::matches_any;

use crate::lex::{Token, TokenKind};

/// True when `line` is a valid `case word in` header.
#[must_use]
pub fn line_opens_case(line: &str) -> bool {
    let mut tokens = Vec::new();
    if crate::lex::tokenize_into(line, &mut tokens).is_err() {
        return false;
    }
    parse_header(line, &tokens).is_some()
}

/// First token is `case`.
#[must_use]
pub fn starts_with_case(source: &str, tokens: &[Token]) -> bool {
    matches!(
        tokens.first(),
        Some(t) if t.kind == TokenKind::Word && t.lexeme(source) == "case"
    )
}

/// Lone `esac` keyword.
#[must_use]
pub fn is_esac_line(line: &str) -> bool {
    line.trim() == "esac"
}

/// stderr message for an incomplete / invalid case line.
#[must_use]
pub fn case_syntax_message(_source: &str, _tokens: &[Token]) -> &'static str {
    "case: Syntax Error."
}
