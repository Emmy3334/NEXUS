//! Syntax analysis for a simple command (Dragon Book Ch. 4, minimal).
//!
//! Minishell1 has no pipes or redirections yet: a line is one argv vector.

use crate::lex::Token;

/// A simple command: program name plus arguments (no operators).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommand<'a> {
    pub argv: Vec<&'a str>,
}

/// Build a borrowing [`SimpleCommand`] from word tokens spanning `source`.
///
/// Returns `None` when there are no tokens (blank input should be filtered
/// before this is called).
pub fn parse_simple<'a>(source: &'a str, tokens: &[Token]) -> Option<SimpleCommand<'a>> {
    if tokens.is_empty() {
        return None;
    }

    let argv = tokens.iter().map(|token| token.lexeme(source)).collect();
    Some(SimpleCommand { argv })
}

/// Fill `argv` from `tokens`, reusing each `String`'s capacity when possible.
///
/// Owned slots avoid tying argv lifetimes to the line buffer across REPL
/// iterations (hot path: clear + `push_str`, no fresh `String` per word when
/// capacity already fits).
pub fn fill_argv(source: &str, tokens: &[Token], argv: &mut Vec<String>) {
    while argv.len() < tokens.len() {
        argv.push(String::new());
    }
    argv.truncate(tokens.len());

    for (slot, token) in argv.iter_mut().zip(tokens.iter()) {
        slot.clear();
        slot.push_str(token.lexeme(source));
    }
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

    #[test]
    fn empty_tokens_yield_none() {
        assert!(parse_simple("", &[]).is_none());
        let tokens = tokens_of("   ");
        assert!(tokens.is_empty());
        assert!(parse_simple("   ", &tokens).is_none());
    }

    #[test]
    fn words_become_argv() {
        let source = "ls -l /tmp";
        let tokens = tokens_of(source);
        let command = parse_simple(source, &tokens).unwrap();
        assert_eq!(command.argv, vec!["ls", "-l", "/tmp"]);
    }

    #[test]
    fn fill_argv_reuses_string_capacity() {
        let mut argv = Vec::new();
        let source = "one two";
        fill_argv(source, &tokens_of(source), &mut argv);
        assert_eq!(argv, ["one", "two"]);
        let capacity = argv[0].capacity();

        let source = "aaa bbb";
        fill_argv(source, &tokens_of(source), &mut argv);
        assert_eq!(argv, ["aaa", "bbb"]);
        assert!(argv[0].capacity() >= capacity);
    }
}
