//! Syntax analysis for a simple command (Dragon Book Ch. 4, minimal).
//!
//! This slice builds argv from [`TokenKind::Word`] only. Lists, pipes, and
//! redirections are tokenized by the lexer but parsed in a later Minishell2
//! slice.

use crate::lex::{Token, TokenKind};

/// A simple command: program name plus arguments (no operators).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommand<'a> {
    pub argv: Vec<&'a str>,
}

/// Build a borrowing [`SimpleCommand`] from word tokens spanning `source`.
///
/// Operator tokens are ignored. Returns `None` when there are no word
/// tokens (blank input should be filtered before this is called).
pub fn parse_simple<'a>(source: &'a str, tokens: &[Token]) -> Option<SimpleCommand<'a>> {
    let argv: Vec<&'a str> = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Word)
        .map(|token| token.lexeme(source))
        .collect();
    if argv.is_empty() {
        None
    } else {
        Some(SimpleCommand { argv })
    }
}

/// Fill `argv` from word `tokens`, reusing each `String`'s capacity when possible.
///
/// Operator tokens are skipped. Owned slots avoid tying argv lifetimes to the
/// line buffer across REPL iterations (hot path: clear + `push_str`, no fresh
/// `String` per word when capacity already fits).
pub fn fill_argv(source: &str, tokens: &[Token], argv: &mut Vec<String>) {
    let word_count = tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Word)
        .count();

    while argv.len() < word_count {
        argv.push(String::new());
    }
    argv.truncate(word_count);

    let mut slot_index = 0;
    for token in tokens {
        if token.kind != TokenKind::Word {
            continue;
        }
        let slot = &mut argv[slot_index];
        slot.clear();
        slot.push_str(token.lexeme(source));
        slot_index += 1;
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

    #[test]
    fn word_only_argv_skips_operators() {
        let source = "ls | wc ; pwd";
        let tokens = tokens_of(source);
        let command = parse_simple(source, &tokens).unwrap();
        assert_eq!(command.argv, vec!["ls", "wc", "pwd"]);

        let mut argv = Vec::new();
        fill_argv(source, &tokens, &mut argv);
        assert_eq!(argv, ["ls", "wc", "pwd"]);
    }

    #[test]
    fn operators_only_yield_no_simple_command() {
        let source = "; | >>";
        let tokens = tokens_of(source);
        assert!(parse_simple(source, &tokens).is_none());
        let mut argv = Vec::new();
        fill_argv(source, &tokens, &mut argv);
        assert!(argv.is_empty());
    }
}
