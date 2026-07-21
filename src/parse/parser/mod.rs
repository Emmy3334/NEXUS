//! Grammar-level parsing: turns a token stream into the [`super`] AST.
//!
//! Token-cursor primitives (peek/consume/advance) live in [`cursor`].
//! Subshell / command dispatch lives in [`command`].

mod command;
mod cursor;

use super::{CommandList, ParseError, Pipeline, Redirect, RedirectKind, SimpleCommand};
use crate::lex::{Token, TokenKind};

pub(super) struct Parser<'src, 'tok> {
    source: &'src str,
    tokens: &'tok [Token],
    index: usize,
}

impl<'src, 'tok> Parser<'src, 'tok> {
    pub(super) fn new(source: &'src str, tokens: &'tok [Token]) -> Self {
        Self {
            source,
            tokens,
            index: 0,
        }
    }

    pub(super) fn parse_list(&mut self) -> Result<Option<CommandList<'src>>, ParseError> {
        self.parse_list_stopping_at(None)
    }

    /// Parse a list, stopping before `stop` (e.g. `)` inside a subshell) if set.
    pub(super) fn parse_list_stopping_at(
        &mut self,
        stop: Option<TokenKind>,
    ) -> Result<Option<CommandList<'src>>, ParseError> {
        let mut pipelines = Vec::new();

        loop {
            self.skip_semicolons();
            if self.is_at_end() || self.peek_kind() == stop {
                break;
            }
            if self.peek_kind() == Some(TokenKind::Pipe)
                || self.peek_kind() == Some(TokenKind::Ampersand)
            {
                return Err(ParseError::NullCommand);
            }
            if !self.can_start_command() {
                return Err(ParseError::UnexpectedToken);
            }

            pipelines.push(self.parse_pipeline()?);
            if self.consume(TokenKind::Ampersand) {
                if let Some(last) = pipelines.last_mut() {
                    last.background = true;
                }
                continue;
            }
            if self.consume(TokenKind::Semicolon) {
                continue;
            }
            if self.is_at_end() || self.peek_kind() == stop {
                break;
            }
            return self.unexpected_remainder();
        }

        if pipelines.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CommandList { pipelines }))
        }
    }

    fn parse_pipeline(&mut self) -> Result<Pipeline<'src>, ParseError> {
        let mut commands = Vec::new();
        commands.push(self.parse_command()?);

        while self.consume(TokenKind::Pipe) {
            if self.can_start_command() {
                commands.push(self.parse_command()?);
            } else {
                return Err(ParseError::NullCommand);
            }
        }

        Ok(Pipeline {
            commands,
            background: false,
        })
    }

    pub(super) fn parse_simple(&mut self) -> Result<SimpleCommand<'src>, ParseError> {
        let mut argv = Vec::new();
        let mut redirects = Vec::new();

        loop {
            match self.peek_kind() {
                Some(TokenKind::Word) => {
                    let token = self.advance();
                    argv.push(token.lexeme(self.source));
                }
                Some(kind) if is_redirect(kind) => {
                    redirects.push(self.parse_redirect()?);
                }
                _ => break,
            }
        }

        if argv.is_empty() {
            Err(ParseError::NullCommand)
        } else {
            Ok(SimpleCommand { argv, redirects })
        }
    }

    pub(super) fn parse_redirect(&mut self) -> Result<Redirect<'src>, ParseError> {
        let kind = match self.advance().kind {
            TokenKind::RedirectOut => RedirectKind::Write,
            TokenKind::RedirectAppend => RedirectKind::Append,
            TokenKind::RedirectIn => RedirectKind::Read,
            TokenKind::Heredoc => RedirectKind::Heredoc,
            _ => return Err(ParseError::UnexpectedToken),
        };

        if self.peek_kind() != Some(TokenKind::Word) {
            return Err(ParseError::MissingRedirectTarget);
        }
        let path = self.advance().lexeme(self.source);
        Ok(Redirect { kind, path })
    }
}

pub(super) const fn is_redirect(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::RedirectOut
            | TokenKind::RedirectAppend
            | TokenKind::RedirectIn
            | TokenKind::Heredoc
    )
}
