//! Integration tests for the parser.

use nexus::lex::{tokenize_into, Token};
use nexus::parse::{fill_argv, parse_line, CommandList, ParseError, SimpleCommand};

fn tokens_of(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens);
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
