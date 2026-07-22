//! Parse `if ( expr… ) then` and `else if ( expr… ) then`.

use crate::lex::{Token, TokenKind};

/// Header of an `if` / `else if` branch (body collected separately).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfHeader {
    /// Raw expression pieces inside `( … )`.
    pub expr: Vec<String>,
}

/// Parse `if ( expr ) then`, or `None` if this is not one.
#[must_use]
pub fn parse_header(source: &str, tokens: &[Token]) -> Option<IfHeader> {
    parse_keyword_expr_then(source, tokens, "if")
}

/// Parse `else if ( expr ) then`, or `None` if this is not one.
#[must_use]
pub fn parse_else_if(source: &str, tokens: &[Token]) -> Option<IfHeader> {
    let mut i = 0;
    if !is_word(source, tokens, i, "else") {
        return None;
    }
    i += 1;
    if !is_word(source, tokens, i, "if") {
        return None;
    }
    i += 1;
    parse_paren_expr_then(source, tokens, i)
}

fn parse_keyword_expr_then(source: &str, tokens: &[Token], kw: &str) -> Option<IfHeader> {
    let mut i = 0;
    if !is_word(source, tokens, i, kw) {
        return None;
    }
    i += 1;
    parse_paren_expr_then(source, tokens, i)
}

fn parse_paren_expr_then(source: &str, tokens: &[Token], mut i: usize) -> Option<IfHeader> {
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
    if !is_word(source, tokens, i, "then") {
        return None;
    }
    i += 1;
    if i != tokens.len() || expr.is_empty() {
        return None;
    }
    Some(IfHeader { expr })
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
        TokenKind::AndAnd => "&&".to_owned(),
        TokenKind::Pipe => "|".to_owned(),
        TokenKind::OrOr => "||".to_owned(),
        TokenKind::LParen => "(".to_owned(),
        TokenKind::RParen => return None,
        _ => return None,
    };
    *index += 1;
    Some(piece)
}
