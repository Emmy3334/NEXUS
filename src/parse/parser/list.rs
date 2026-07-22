//! List-level parsing: `;` / `&` / `&&` / `||` between pipelines.

use super::Parser;
use crate::lex::TokenKind;
use crate::parse::{CommandList, ParseError, Pipeline, PipelineJoin};

impl<'src, 'tok> Parser<'src, 'tok> {
    /// Parse a list, stopping before `stop` (e.g. `)` inside a subshell) if set.
    pub(super) fn parse_list_stopping_at(
        &mut self,
        stop: Option<TokenKind>,
    ) -> Result<Option<CommandList<'src>>, ParseError> {
        let mut pipelines = Vec::new();
        let mut pending_join = PipelineJoin::Seq;

        loop {
            match self.next_pipeline(stop, pending_join)? {
                None => break,
                Some((pipeline, next_join)) => {
                    pipelines.push(pipeline);
                    pending_join = next_join;
                }
            }
        }

        if pipelines.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CommandList { pipelines }))
        }
    }

    fn next_pipeline(
        &mut self,
        stop: Option<TokenKind>,
        pending_join: PipelineJoin,
    ) -> Result<Option<(Pipeline<'src>, PipelineJoin)>, ParseError> {
        self.skip_semicolons();
        if self.is_at_end() || self.peek_kind() == stop {
            if matches!(pending_join, PipelineJoin::And | PipelineJoin::Or) {
                return Err(ParseError::NullCommand);
            }
            return Ok(None);
        }
        if is_null_list_op(self.peek_kind()) {
            return Err(ParseError::NullCommand);
        }
        if !self.can_start_command() {
            return Err(ParseError::UnexpectedToken);
        }

        let mut pipeline = self.parse_pipeline()?;
        pipeline.join = pending_join;
        let next_join = match after_pipeline(self, stop)? {
            After::End => PipelineJoin::Seq,
            After::More {
                join,
                background_prev,
            } => {
                if background_prev {
                    pipeline.background = true;
                }
                join
            }
        };
        Ok(Some((pipeline, next_join)))
    }
}

enum After {
    End,
    More {
        join: PipelineJoin,
        background_prev: bool,
    },
}

fn after_pipeline(
    parser: &mut Parser<'_, '_>,
    stop: Option<TokenKind>,
) -> Result<After, ParseError> {
    if parser.consume(TokenKind::Ampersand) {
        return Ok(After::More {
            join: PipelineJoin::Seq,
            background_prev: true,
        });
    }
    if parser.consume(TokenKind::Semicolon) {
        return Ok(After::More {
            join: PipelineJoin::Seq,
            background_prev: false,
        });
    }
    if parser.consume(TokenKind::AndAnd) {
        return Ok(After::More {
            join: PipelineJoin::And,
            background_prev: false,
        });
    }
    if parser.consume(TokenKind::OrOr) {
        return Ok(After::More {
            join: PipelineJoin::Or,
            background_prev: false,
        });
    }
    if parser.is_at_end() || parser.peek_kind() == stop {
        return Ok(After::End);
    }
    parser.unexpected_remainder().map(|_| After::End)
}

const fn is_null_list_op(kind: Option<TokenKind>) -> bool {
    matches!(
        kind,
        Some(TokenKind::Pipe)
            | Some(TokenKind::Ampersand)
            | Some(TokenKind::AndAnd)
            | Some(TokenKind::OrOr)
    )
}
