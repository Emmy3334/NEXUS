//! Syntax analysis for command lists and pipelines (Dragon Book Ch. 4).
//!
//! Grammar (Minishell2, no redirections yet):
//! ```text
//! line     → list
//! list     → pipeline ( ';' pipeline )* [ ';' ]
//! pipeline → simple ( '|' simple )*
//! simple   → WORD+
//! ```
//!
//! Redirect tokens (`>`, `<`, `>>`, `<<`) are rejected until the redirection
//! slice. Empty commands around `|` are errors; a trailing `;` is allowed.

use crate::lex::{Token, TokenKind};

/// A sequence of pipelines separated by `;`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandList<'a> {
    pub pipelines: Vec<Pipeline<'a>>,
}

impl<'a> CommandList<'a> {
    /// A list that is exactly one simple command (no `;` or `|`).
    #[must_use]
    pub fn as_single_command(&self) -> Option<&SimpleCommand<'a>> {
        match self.pipelines.as_slice() {
            [pipeline] if pipeline.commands.len() == 1 => Some(&pipeline.commands[0]),
            _ => None,
        }
    }
}

/// One or more simple commands connected by `|`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline<'a> {
    pub commands: Vec<SimpleCommand<'a>>,
}

/// A simple command: program name plus arguments (no operators).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommand<'a> {
    pub argv: Vec<&'a str>,
}

/// Why `parse_line` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Missing command where one is required (e.g. leading/trailing `|`).
    NullCommand,
    /// Redirect operator before the redirection slice is implemented.
    RedirectNotImplemented,
    /// Token that cannot start or continue the current construct.
    UnexpectedToken,
}

impl ParseError {
    /// Human-readable message for stderr.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::NullCommand => "Invalid null command.",
            Self::RedirectNotImplemented => "nexus: redirections are not implemented yet.",
            Self::UnexpectedToken => "Syntax error.",
        }
    }
}

/// Parse `tokens` spanning `source` into a command list.
///
/// Returns `Ok(None)` when there is nothing to run (empty input or only `;`).
pub fn parse_line<'a>(
    source: &'a str,
    tokens: &[Token],
) -> Result<Option<CommandList<'a>>, ParseError> {
    let mut parser = Parser {
        source,
        tokens,
        index: 0,
    };
    parser.parse_list()
}

/// Fill `argv` from word slices, reusing each `String`'s capacity when possible.
///
/// Owned slots avoid tying argv lifetimes to the line buffer across REPL
/// iterations (hot path: clear + `push_str`, no fresh `String` per word when
/// capacity already fits).
pub fn fill_argv(words: &[&str], argv: &mut Vec<String>) {
    while argv.len() < words.len() {
        argv.push(String::new());
    }
    argv.truncate(words.len());

    for (slot, word) in argv.iter_mut().zip(words.iter()) {
        slot.clear();
        slot.push_str(word);
    }
}

struct Parser<'src, 'tok> {
    source: &'src str,
    tokens: &'tok [Token],
    index: usize,
}

impl<'src, 'tok> Parser<'src, 'tok> {
    fn parse_list(&mut self) -> Result<Option<CommandList<'src>>, ParseError> {
        let mut pipelines = Vec::new();

        loop {
            self.skip_semicolons();
            if self.is_at_end() {
                break;
            }
            if self.peek_kind() == Some(TokenKind::Pipe) {
                return Err(ParseError::NullCommand);
            }
            if self.peek_kind().is_some_and(is_redirect) {
                return Err(ParseError::RedirectNotImplemented);
            }
            if self.peek_kind() != Some(TokenKind::Word) {
                return Err(ParseError::UnexpectedToken);
            }

            pipelines.push(self.parse_pipeline()?);
            if self.consume(TokenKind::Semicolon) {
                continue;
            }
            if self.is_at_end() {
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
        commands.push(self.parse_simple()?);

        while self.consume(TokenKind::Pipe) {
            match self.peek_kind() {
                Some(TokenKind::Word) => commands.push(self.parse_simple()?),
                Some(kind) if is_redirect(kind) => {
                    return Err(ParseError::RedirectNotImplemented);
                }
                _ => return Err(ParseError::NullCommand),
            }
        }

        Ok(Pipeline { commands })
    }

    fn parse_simple(&mut self) -> Result<SimpleCommand<'src>, ParseError> {
        let mut argv = Vec::new();
        while self.peek_kind() == Some(TokenKind::Word) {
            let token = self.advance();
            argv.push(token.lexeme(self.source));
        }
        if argv.is_empty() {
            Err(ParseError::NullCommand)
        } else {
            Ok(SimpleCommand { argv })
        }
    }

    fn unexpected_remainder(&self) -> Result<Option<CommandList<'src>>, ParseError> {
        match self.peek_kind() {
            Some(kind) if is_redirect(kind) => Err(ParseError::RedirectNotImplemented),
            Some(TokenKind::Pipe) => Err(ParseError::NullCommand),
            _ => Err(ParseError::UnexpectedToken),
        }
    }

    fn skip_semicolons(&mut self) {
        while self.consume(TokenKind::Semicolon) {}
    }

    fn is_at_end(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.index).map(|token| token.kind)
    }

    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == Some(kind) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.index];
        self.index += 1;
        token
    }
}

const fn is_redirect(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::RedirectOut
            | TokenKind::RedirectAppend
            | TokenKind::RedirectIn
            | TokenKind::Heredoc
    )
}
