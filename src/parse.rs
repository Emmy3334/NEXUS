//! Syntax analysis for command lists and pipelines (Dragon Book Ch. 4).
//!
//! Grammar (Minishell2):
//! ```text
//! line     → list
//! list     → pipeline ( ';' pipeline )* [ ';' ]
//! pipeline → simple ( '|' simple )*
//! simple   → ( WORD | redirect )+   # at least one WORD
//! redirect → ( '>' | '<' | '>>' | '<<' ) WORD
//! ```
//!
//! Heredoc body lines are collected by the REPL after parse (not on this line).
//! Empty commands around `|` are errors; a trailing `;` is allowed.

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

/// A simple command: argv plus optional file / heredoc redirections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommand<'a> {
    pub argv: Vec<&'a str>,
    pub redirects: Vec<Redirect<'a>>,
}

/// One redirection attached to a simple command.
///
/// For file redirects, [`Redirect::path`] is the file path. For heredoc,
/// it is the end delimiter (body is collected later from the input stream).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect<'a> {
    pub kind: RedirectKind,
    pub path: &'a str,
}

/// Redirection operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectKind {
    /// `<` — stdin from file.
    Read,
    /// `>` — stdout truncate/create.
    Write,
    /// `>>` — stdout append.
    Append,
    /// `<<` — stdin from heredoc body until `path` delimiter line.
    Heredoc,
}

/// Why `parse_line` failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Missing command where one is required (e.g. leading/trailing `|`).
    NullCommand,
    /// Redirect operator without a following word.
    MissingRedirectTarget,
    /// Token that cannot start or continue the current construct.
    UnexpectedToken,
}

impl ParseError {
    /// Human-readable message for stderr.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::NullCommand => "Invalid null command.",
            Self::MissingRedirectTarget => "Missing name for redirect.",
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

/// Fill `argv` from raw word lexemes: quotes/`$` expansion, then pathname globs.
///
/// One raw word may expand to multiple argv entries. Reuses `argv` capacity
/// when the result fits in existing slots.
pub fn fill_argv(
    words: &[&str],
    argv: &mut Vec<String>,
    env: &crate::env::ShellEnvironment,
    last_status: u8,
) -> Result<(), crate::lex::LexError> {
    let mut expanded = Vec::new();
    let mut word = crate::expand::ExpandedWord::default();
    for raw in words {
        crate::expand::expand_word_for_exec_into(raw, env, last_status, &mut word)?;
        expanded.extend(crate::glob::expand_globs(&word));
    }

    while argv.len() < expanded.len() {
        argv.push(String::new());
    }
    argv.truncate(expanded.len());
    for (slot, value) in argv.iter_mut().zip(expanded) {
        *slot = value;
    }
    Ok(())
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
            if !self.can_start_simple() {
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
            if self.can_start_simple() {
                commands.push(self.parse_simple()?);
            } else {
                return Err(ParseError::NullCommand);
            }
        }

        Ok(Pipeline { commands })
    }

    fn parse_simple(&mut self) -> Result<SimpleCommand<'src>, ParseError> {
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

    fn parse_redirect(&mut self) -> Result<Redirect<'src>, ParseError> {
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

    fn can_start_simple(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Word)
                | Some(TokenKind::RedirectOut)
                | Some(TokenKind::RedirectAppend)
                | Some(TokenKind::RedirectIn)
                | Some(TokenKind::Heredoc)
        )
    }

    fn unexpected_remainder(&self) -> Result<Option<CommandList<'src>>, ParseError> {
        match self.peek_kind() {
            Some(kind) if is_redirect(kind) => Err(ParseError::UnexpectedToken),
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
