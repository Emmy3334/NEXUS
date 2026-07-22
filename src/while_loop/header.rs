//! Parse `while ( expr… )` from an already-tokenized line.

use crate::lex::{Token, TokenKind};

/// Header of a `while` loop (body collected separately until `end`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileHeader {
    /// Raw expression pieces inside `( … )` (expanded each iteration).
    pub expr: Vec<String>,
}

/// Parse a complete while header line, or `None` if this is not one.
#[must_use]
pub fn parse_header(source: &str, tokens: &[Token]) -> Option<WhileHeader> {
    let mut i = 0;
    if !is_word(source, tokens, i, "while") {
        return None;
    }
    i += 1;
    if tokens.get(i).map(|t| t.kind) != Some(TokenKind::LParen) {
        return None;
    }
    i += 1;
    let mut expr = Vec::new();
    while let Some(piece) = take_expr_piece(source, tokens, &mut i) {
        expr.push(piece);
    }
    if tokens.get(i).map(|t| t.kind) != Some(TokenKind::RParen) {
        return None;
    }
    i += 1;
    if i != tokens.len() || expr.is_empty() {
        return None;
    }
    Some(WhileHeader { expr })
}

fn is_word(source: &str, tokens: &[Token], index: usize, want: &str) -> bool {
    matches!(
        tokens.get(index),
        Some(t) if t.kind == TokenKind::Word && t.lexeme(source) == want
    )
}

fn take_expr_piece(source: &str, tokens: &[Token], index: &mut usize) -> Option<String> {
    let token = tokens.get(*index)?;
    let piece = match token.kind {
        TokenKind::Word => token.lexeme(source).to_owned(),
        TokenKind::RedirectIn => "<".to_owned(),
        TokenKind::RedirectOut => ">".to_owned(),
        TokenKind::RedirectAppend => ">>".to_owned(),
        TokenKind::Ampersand => "&".to_owned(),
        TokenKind::Pipe => "|".to_owned(),
        TokenKind::LParen => "(".to_owned(),
        TokenKind::RParen => return None,
        _ => return None,
    };
    *index += 1;
    Some(piece)
}
