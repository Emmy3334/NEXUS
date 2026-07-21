//! Tests for file-redirection handling (apply lands in a later slice).

use nexus::parse::{parse_line, ParseError};

#[test]
fn redirects_still_rejected_at_parse() {
    let mut tokens = Vec::new();
    nexus::lex::tokenize_into("ls > out", &mut tokens);
    assert_eq!(
        parse_line("ls > out", &tokens).unwrap_err(),
        ParseError::RedirectNotImplemented
    );

    nexus::lex::tokenize_into("cat < in", &mut tokens);
    assert_eq!(
        parse_line("cat < in", &tokens).unwrap_err(),
        ParseError::RedirectNotImplemented
    );

    nexus::lex::tokenize_into("cmd >> log", &mut tokens);
    assert_eq!(
        parse_line("cmd >> log", &tokens).unwrap_err(),
        ParseError::RedirectNotImplemented
    );
}
