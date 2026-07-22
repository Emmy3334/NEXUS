//! Token-cursor primitives shared by the grammar-level parse methods.

use super::Parser;
use crate::lex::{Token, TokenKind};
use crate::parse::{CommandList, ParseError};

impl<'src, 'tok> Parser<'src, 'tok> {
    pub(super) fn can_start_command(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Word)
                | Some(TokenKind::LParen)
                | Some(TokenKind::RedirectOut)
                | Some(TokenKind::RedirectAppend)
                | Some(TokenKind::RedirectIn)
                | Some(TokenKind::Heredoc)
        )
    }

    pub(super) fn unexpected_remainder(&self) -> Result<Option<CommandList<'src>>, ParseError> {
        match self.peek_kind() {
            Some(kind) if super::is_redirect(kind) => Err(ParseError::UnexpectedToken),
            Some(TokenKind::Pipe) => Err(ParseError::NullCommand),
            Some(TokenKind::Ampersand) => Err(ParseError::NullCommand),
            Some(TokenKind::AndAnd) | Some(TokenKind::OrOr) => Err(ParseError::NullCommand),
            Some(TokenKind::RParen) => Err(ParseError::UnexpectedToken),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    pub(super) fn skip_semicolons(&mut self) {
        while self.consume(TokenKind::Semicolon) {}
    }

    pub(super) fn is_at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    pub(super) fn peek_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.index).map(|token| token.kind)
    }

    pub(super) fn consume(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == Some(kind) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    pub(super) fn advance(&mut self) -> Token {
        let token = self.tokens[self.index];
        self.index += 1;
        token
    }
}
