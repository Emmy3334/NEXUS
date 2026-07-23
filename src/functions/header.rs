//! Detect `name() {` / `function name {` headers.

use crate::lex::{Token, TokenKind};

use super::body;
use super::define::FunctionHeader;

/// Parse a function definition header from a line and its tokens.
#[must_use]
pub fn parse_header(expanded: &str, tokens: &[Token]) -> Option<FunctionHeader> {
    let (name, brace_at) = match_header(expanded, tokens)?;
    let after = expanded.get(brace_at + 1..).unwrap_or("");
    match body::split_closed(after) {
        Some((inner, _)) => Some(FunctionHeader::Complete {
            name,
            body: inner.trim().to_owned(),
        }),
        None => Some(FunctionHeader::Open {
            name,
            first: after.to_owned(),
        }),
    }
}

fn match_header(expanded: &str, tokens: &[Token]) -> Option<(String, usize)> {
    if tokens.is_empty() {
        return None;
    }
    if is_word(tokens[0], expanded, "function") {
        return match_function_kw(expanded, tokens);
    }
    match_name_parens(expanded, tokens, 0)
}

fn match_function_kw(expanded: &str, tokens: &[Token]) -> Option<(String, usize)> {
    let name_tok = tokens.get(1)?;
    if name_tok.kind != TokenKind::Word {
        return None;
    }
    let name = name_tok.lexeme(expanded).to_owned();
    if !is_ident(&name) {
        return None;
    }
    let mut i = 2;
    if tokens.get(i).is_some_and(|t| t.kind == TokenKind::LParen)
        && tokens
            .get(i + 1)
            .is_some_and(|t| t.kind == TokenKind::RParen)
    {
        i += 2;
    }
    let idx = brace_byte_index(expanded, tokens.get(i)?)?;
    Some((name, idx))
}

fn match_name_parens(expanded: &str, tokens: &[Token], start: usize) -> Option<(String, usize)> {
    let name_tok = tokens.get(start)?;
    if name_tok.kind != TokenKind::Word {
        return None;
    }
    let name = name_tok.lexeme(expanded).to_owned();
    if !is_ident(&name) {
        return None;
    }
    if tokens.get(start + 1)?.kind != TokenKind::LParen {
        return None;
    }
    if tokens.get(start + 2)?.kind != TokenKind::RParen {
        return None;
    }
    let idx = brace_byte_index(expanded, tokens.get(start + 3)?)?;
    Some((name, idx))
}

fn brace_byte_index(expanded: &str, tok: &Token) -> Option<usize> {
    if tok.kind != TokenKind::Word || tok.lexeme(expanded) != "{" {
        return None;
    }
    Some(tok.start)
}

fn is_word(tok: Token, expanded: &str, want: &str) -> bool {
    tok.kind == TokenKind::Word && tok.lexeme(expanded) == want
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
