//! Parse `case word in`.

use crate::lex::{Token, TokenKind};

/// Header of a `case` block (arms collected separately).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseHeader {
    /// Raw subject word (expanded at run time).
    pub subject: String,
}

/// Parse `case word in`, or `None` if this is not one.
#[must_use]
pub fn parse_header(source: &str, tokens: &[Token]) -> Option<CaseHeader> {
    if tokens.len() != 3 {
        return None;
    }
    if !is_word(source, tokens, 0, "case") {
        return None;
    }
    if tokens[1].kind != TokenKind::Word {
        return None;
    }
    if !is_word(source, tokens, 2, "in") {
        return None;
    }
    Some(CaseHeader {
        subject: tokens[1].lexeme(source).to_owned(),
    })
}

fn is_word(source: &str, tokens: &[Token], index: usize, want: &str) -> bool {
    matches!(
        tokens.get(index),
        Some(t) if t.kind == TokenKind::Word && t.lexeme(source) == want
    )
}
