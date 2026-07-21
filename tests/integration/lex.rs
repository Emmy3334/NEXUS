//! Integration tests for the lexer.

use nexus::lex::{tokenize_into, Token, TokenKind};

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens);
    tokens
}

fn lexemes(source: &str) -> Vec<&str> {
    tokenize(source)
        .iter()
        .map(|token| token.lexeme(source))
        .collect()
}

fn kinds(source: &str) -> Vec<TokenKind> {
    tokenize(source).iter().map(|token| token.kind).collect()
}

#[test]
fn empty_and_whitespace_only_yield_no_tokens() {
    assert!(tokenize("").is_empty());
    assert!(tokenize("   \t  ").is_empty());
}

#[test]
fn single_word() {
    assert_eq!(lexemes("ls"), vec!["ls"]);
    assert_eq!(lexemes("  ls  "), vec!["ls"]);
}

#[test]
fn multiple_words_collapse_whitespace() {
    assert_eq!(lexemes("ls -l /tmp"), vec!["ls", "-l", "/tmp"]);
    assert_eq!(
        lexemes("echo   hello\tworld"),
        vec!["echo", "hello", "world"]
    );
}

#[test]
fn tokenize_into_reuses_buffer() {
    let mut tokens = Vec::with_capacity(4);
    tokenize_into("one two", &mut tokens);
    assert_eq!(tokens.len(), 2);
    let capacity_after_first = tokens.capacity();

    tokenize_into("a b c", &mut tokens);
    assert_eq!(
        tokens
            .iter()
            .map(|token| token.lexeme("a b c"))
            .collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
    assert!(tokens.capacity() >= capacity_after_first);
}

#[test]
fn words_are_non_empty_and_whitespace_free() {
    let source = "  cmd  -a  ./path  ";
    let tokens = tokenize(source);
    for token in &tokens {
        let lexeme = token.lexeme(source);
        assert!(!lexeme.is_empty());
        assert!(!lexeme.chars().any(char::is_whitespace));
        assert_eq!(token.kind, TokenKind::Word);
    }
}

#[test]
fn try_lexeme_rejects_out_of_bounds() {
    let token = Token {
        kind: TokenKind::Word,
        start: 0,
        end: 99,
    };
    assert!(token.try_lexeme("hi").is_none());
    assert_eq!(
        Token {
            kind: TokenKind::Word,
            start: 0,
            end: 2,
        }
        .try_lexeme("hi"),
        Some("hi")
    );
}

#[test]
fn semicolon_and_pipe() {
    assert_eq!(
        kinds("ls ; pwd"),
        vec![TokenKind::Word, TokenKind::Semicolon, TokenKind::Word]
    );
    assert_eq!(lexemes("ls ; pwd"), vec!["ls", ";", "pwd"]);
    assert_eq!(
        kinds("ls|wc"),
        vec![TokenKind::Word, TokenKind::Pipe, TokenKind::Word]
    );
    assert_eq!(lexemes("ls|wc"), vec!["ls", "|", "wc"]);
}

#[test]
fn single_and_double_redirects() {
    assert_eq!(
        kinds("cat < in > out"),
        vec![
            TokenKind::Word,
            TokenKind::RedirectIn,
            TokenKind::Word,
            TokenKind::RedirectOut,
            TokenKind::Word,
        ]
    );
    assert_eq!(
        kinds("cmd >> log << END"),
        vec![
            TokenKind::Word,
            TokenKind::RedirectAppend,
            TokenKind::Word,
            TokenKind::Heredoc,
            TokenKind::Word,
        ]
    );
    assert_eq!(lexemes("a>>b<<c"), vec!["a", ">>", "b", "<<", "c"]);
}

#[test]
fn redirects_prefer_two_char_over_one() {
    assert_eq!(kinds(">>"), vec![TokenKind::RedirectAppend]);
    assert_eq!(
        kinds("> >"),
        vec![TokenKind::RedirectOut, TokenKind::RedirectOut]
    );
    assert_eq!(kinds("<<"), vec![TokenKind::Heredoc]);
    assert_eq!(
        kinds("< <"),
        vec![TokenKind::RedirectIn, TokenKind::RedirectIn]
    );
}

#[test]
fn adjacent_operators_without_spaces() {
    assert_eq!(
        kinds("ls;pwd|wc"),
        vec![
            TokenKind::Word,
            TokenKind::Semicolon,
            TokenKind::Word,
            TokenKind::Pipe,
            TokenKind::Word,
        ]
    );
    assert_eq!(lexemes("a>b<c"), vec!["a", ">", "b", "<", "c"]);
}

#[test]
fn operator_kinds_report_is_operator() {
    assert!(!TokenKind::Word.is_operator());
    assert!(TokenKind::Semicolon.is_operator());
    assert!(TokenKind::Pipe.is_operator());
    assert!(TokenKind::RedirectOut.is_operator());
    assert!(TokenKind::RedirectAppend.is_operator());
    assert!(TokenKind::RedirectIn.is_operator());
    assert!(TokenKind::Heredoc.is_operator());
}
