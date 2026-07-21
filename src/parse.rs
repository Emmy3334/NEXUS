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

    /// Whether any pipeline uses `|` (multi-command).
    #[must_use]
    pub fn contains_pipe(&self) -> bool {
        self.pipelines
            .iter()
            .any(|pipeline| pipeline.commands.len() > 1)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex;

    fn tokens_of(source: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        lex::tokenize_into(source, &mut tokens);
        tokens
    }

    fn parse(source: &str) -> Result<Option<CommandList<'_>>, ParseError> {
        parse_line(source, &tokens_of(source))
    }

    fn argv_of<'a>(command: &SimpleCommand<'a>) -> Vec<&'a str> {
        command.argv.clone()
    }

    #[test]
    fn empty_input_yields_none() {
        assert_eq!(parse("").unwrap(), None);
        assert_eq!(parse("   ").unwrap(), None);
        assert_eq!(parse(";").unwrap(), None);
        assert_eq!(parse(";;;").unwrap(), None);
    }

    #[test]
    fn single_simple_command() {
        let list = parse("ls -l /tmp").unwrap().unwrap();
        assert_eq!(
            argv_of(list.as_single_command().unwrap()),
            vec!["ls", "-l", "/tmp"]
        );
    }

    #[test]
    fn semicolon_separates_pipelines() {
        let list = parse("ls ; pwd").unwrap().unwrap();
        assert!(list.as_single_command().is_none());
        assert_eq!(list.pipelines.len(), 2);
        assert_eq!(argv_of(&list.pipelines[0].commands[0]), vec!["ls"]);
        assert_eq!(argv_of(&list.pipelines[1].commands[0]), vec!["pwd"]);
    }

    #[test]
    fn trailing_semicolon_is_allowed() {
        let list = parse("true;").unwrap().unwrap();
        assert_eq!(argv_of(list.as_single_command().unwrap()), vec!["true"]);
    }

    #[test]
    fn pipe_builds_pipeline() {
        let list = parse("ls -l | wc -l").unwrap().unwrap();
        assert!(list.as_single_command().is_none());
        assert_eq!(list.pipelines.len(), 1);
        let pipeline = &list.pipelines[0];
        assert_eq!(pipeline.commands.len(), 2);
        assert_eq!(argv_of(&pipeline.commands[0]), vec!["ls", "-l"]);
        assert_eq!(argv_of(&pipeline.commands[1]), vec!["wc", "-l"]);
    }

    #[test]
    fn list_of_pipelines() {
        let list = parse("ls | wc ; pwd").unwrap().unwrap();
        assert_eq!(list.pipelines.len(), 2);
        assert_eq!(list.pipelines[0].commands.len(), 2);
        assert_eq!(argv_of(&list.pipelines[1].commands[0]), vec!["pwd"]);
    }

    #[test]
    fn adjacent_operators_parse() {
        let list = parse("ls|wc;pwd").unwrap().unwrap();
        assert_eq!(list.pipelines.len(), 2);
        assert_eq!(list.pipelines[0].commands.len(), 2);
    }

    #[test]
    fn null_command_around_pipe_is_error() {
        assert_eq!(parse("| wc").unwrap_err(), ParseError::NullCommand);
        assert_eq!(parse("ls |").unwrap_err(), ParseError::NullCommand);
        assert_eq!(parse("ls || wc").unwrap_err(), ParseError::NullCommand);
    }

    #[test]
    fn redirects_rejected_until_later_slice() {
        assert_eq!(
            parse("ls > out").unwrap_err(),
            ParseError::RedirectNotImplemented
        );
        assert_eq!(
            parse("cat < in").unwrap_err(),
            ParseError::RedirectNotImplemented
        );
        assert_eq!(
            parse("cmd >> log").unwrap_err(),
            ParseError::RedirectNotImplemented
        );
        assert_eq!(
            parse("cmd << END").unwrap_err(),
            ParseError::RedirectNotImplemented
        );
    }

    #[test]
    fn fill_argv_reuses_string_capacity() {
        let mut argv = Vec::new();
        fill_argv(&["one", "two"], &mut argv);
        assert_eq!(argv, ["one", "two"]);
        let capacity = argv[0].capacity();

        fill_argv(&["aaa", "bbb"], &mut argv);
        assert_eq!(argv, ["aaa", "bbb"]);
        assert!(argv[0].capacity() >= capacity);
    }
}
