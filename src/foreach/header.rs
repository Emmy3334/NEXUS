//! Parse `foreach name ( word… )` from an already-tokenized line.

use crate::lex::{Token, TokenKind};

/// Header of a `foreach` loop (body collected separately until `end`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForEachHeader {
    /// Loop variable name (`$name` each iteration).
    pub var: String,
    /// Raw word lexemes inside `( … )` (expanded at run time).
    pub items: Vec<String>,
}

/// Parse a complete foreach header line, or `None` if this is not one.
#[must_use]
pub fn parse_header(source: &str, tokens: &[Token]) -> Option<ForEachHeader> {
    let mut i = 0;
    if token_word(source, tokens, i)? != "foreach" {
        return None;
    }
    i += 1;
    let var = token_word(source, tokens, i)?.to_owned();
    if !is_ident(&var) {
        return None;
    }
    i += 1;
    if tokens.get(i).map(|t| t.kind) != Some(TokenKind::LParen) {
        return None;
    }
    i += 1;
    let mut items = Vec::new();
    while tokens.get(i).map(|t| t.kind) == Some(TokenKind::Word) {
        items.push(tokens[i].lexeme(source).to_owned());
        i += 1;
    }
    if tokens.get(i).map(|t| t.kind) != Some(TokenKind::RParen) {
        return None;
    }
    i += 1;
    if i != tokens.len() {
        return None;
    }
    Some(ForEachHeader { var, items })
}

fn token_word<'a>(source: &'a str, tokens: &[Token], index: usize) -> Option<&'a str> {
    let token = tokens.get(index)?;
    if token.kind != TokenKind::Word {
        return None;
    }
    Some(token.lexeme(source))
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
