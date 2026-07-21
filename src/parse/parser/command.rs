//! Parse a pipeline stage: simple command or `( list )` subshell.

use super::{is_redirect, Parser};
use crate::lex::TokenKind;
use crate::parse::{ParseError, PipelineCommand};

impl<'src, 'tok> Parser<'src, 'tok> {
    pub(super) fn parse_command(&mut self) -> Result<PipelineCommand<'src>, ParseError> {
        if self.peek_kind() == Some(TokenKind::LParen) {
            self.parse_subshell()
        } else {
            Ok(PipelineCommand::Simple(self.parse_simple()?))
        }
    }

    fn parse_subshell(&mut self) -> Result<PipelineCommand<'src>, ParseError> {
        self.advance(); // '('
        let Some(list) = self.parse_list_stopping_at(Some(TokenKind::RParen))? else {
            return Err(ParseError::NullCommand);
        };
        if !self.consume(TokenKind::RParen) {
            return Err(ParseError::UnexpectedToken);
        }
        let mut redirects = Vec::new();
        while matches!(self.peek_kind(), Some(kind) if is_redirect(kind)) {
            redirects.push(self.parse_redirect()?);
        }
        Ok(PipelineCommand::Subshell { list, redirects })
    }
}
