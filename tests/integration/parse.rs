//! Integration tests for the parser.

use nexus::env::ShellEnvironment;
use nexus::lex::{tokenize_into, Token};
use nexus::parse::{
    fill_argv, parse_line, CommandList, ParseError, PipelineCommand, Redirect, RedirectKind,
};

fn tokens_of(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens).expect("lex ok");
    tokens
}

fn parse(source: &str) -> Result<Option<CommandList<'_>>, ParseError> {
    parse_line(source, &tokens_of(source))
}

fn argv_of<'a>(command: &PipelineCommand<'a>) -> Vec<&'a str> {
    match command {
        PipelineCommand::Simple(cmd) => cmd.argv.clone(),
        PipelineCommand::Subshell { .. } => panic!("expected simple command"),
    }
}

fn simple_of<'a>(command: &'a PipelineCommand<'a>) -> &'a nexus::parse::SimpleCommand<'a> {
    match command {
        PipelineCommand::Simple(cmd) => cmd,
        PipelineCommand::Subshell { .. } => panic!("expected simple command"),
    }
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
    let cmd = list.as_single_command().unwrap();
    assert_eq!(cmd.argv, vec!["ls", "-l", "/tmp"]);
    assert!(cmd.redirects.is_empty());
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
    assert_eq!(list.as_single_command().unwrap().argv, vec!["true"]);
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
    assert_eq!(parse("ls &&").unwrap_err(), ParseError::NullCommand);
    assert_eq!(parse("|| true").unwrap_err(), ParseError::NullCommand);
}

#[test]
fn and_or_join_pipelines() {
    let list = parse("false && true || true").unwrap().unwrap();
    assert_eq!(list.pipelines.len(), 3);
    assert_eq!(list.pipelines[0].join, nexus::parse::PipelineJoin::Seq);
    assert_eq!(list.pipelines[1].join, nexus::parse::PipelineJoin::And);
    assert_eq!(list.pipelines[2].join, nexus::parse::PipelineJoin::Or);
}

#[test]
fn file_redirects_parse_on_simple_command() {
    let list = parse("ls > out").unwrap().unwrap();
    let cmd = list.as_single_command().unwrap();
    assert_eq!(cmd.argv, vec!["ls"]);
    assert_eq!(
        cmd.redirects,
        vec![Redirect {
            kind: RedirectKind::Write,
            path: "out",
        }]
    );

    let list = parse("cat < in >> log").unwrap().unwrap();
    let cmd = list.as_single_command().unwrap();
    assert_eq!(cmd.argv, vec!["cat"]);
    assert_eq!(
        cmd.redirects,
        vec![
            Redirect {
                kind: RedirectKind::Read,
                path: "in",
            },
            Redirect {
                kind: RedirectKind::Append,
                path: "log",
            },
        ]
    );
}

#[test]
fn heredoc_parses_on_simple_command() {
    let list = parse("cat << EOF").unwrap().unwrap();
    let cmd = list.as_single_command().unwrap();
    assert_eq!(cmd.argv, vec!["cat"]);
    assert_eq!(
        cmd.redirects,
        vec![Redirect {
            kind: RedirectKind::Heredoc,
            path: "EOF",
        }]
    );
}

#[test]
fn redirect_before_command_word() {
    let list = parse("> out echo hi").unwrap().unwrap();
    let cmd = list.as_single_command().unwrap();
    assert_eq!(cmd.argv, vec!["echo", "hi"]);
    assert_eq!(cmd.redirects[0].kind, RedirectKind::Write);
    assert_eq!(cmd.redirects[0].path, "out");
}

#[test]
fn redirect_in_pipeline_stage() {
    let list = parse("ls > out | wc").unwrap().unwrap();
    assert_eq!(list.pipelines[0].commands[0].redirects().len(), 1);
    assert_eq!(simple_of(&list.pipelines[0].commands[1]).argv, vec!["wc"]);
}

#[test]
fn parentheses_parse_as_subshell() {
    let list = parse("(ls ; pwd)").unwrap().unwrap();
    assert!(list.as_single_command().is_none());
    match &list.pipelines[0].commands[0] {
        PipelineCommand::Subshell { list, redirects } => {
            assert!(redirects.is_empty());
            assert_eq!(list.pipelines.len(), 2);
            assert_eq!(argv_of(&list.pipelines[0].commands[0]), vec!["ls"]);
            assert_eq!(argv_of(&list.pipelines[1].commands[0]), vec!["pwd"]);
        }
        PipelineCommand::Simple(_) => panic!("expected subshell"),
    }
}

#[test]
fn parentheses_with_group_redirect() {
    let list = parse("(echo hi) > out").unwrap().unwrap();
    match &list.pipelines[0].commands[0] {
        PipelineCommand::Subshell { list, redirects } => {
            assert_eq!(argv_of(&list.pipelines[0].commands[0]), vec!["echo", "hi"]);
            assert_eq!(
                redirects,
                &vec![Redirect {
                    kind: RedirectKind::Write,
                    path: "out",
                }]
            );
        }
        PipelineCommand::Simple(_) => panic!("expected subshell"),
    }
}

#[test]
fn parentheses_in_pipeline() {
    let list = parse("(echo a) | cat").unwrap().unwrap();
    assert_eq!(list.pipelines[0].commands.len(), 2);
    assert!(matches!(
        list.pipelines[0].commands[0],
        PipelineCommand::Subshell { .. }
    ));
    assert_eq!(argv_of(&list.pipelines[0].commands[1]), vec!["cat"]);
}

#[test]
fn empty_or_unbalanced_parentheses_are_errors() {
    assert_eq!(parse("()").unwrap_err(), ParseError::NullCommand);
    assert_eq!(parse("(ls").unwrap_err(), ParseError::UnexpectedToken);
    assert_eq!(parse("ls)").unwrap_err(), ParseError::UnexpectedToken);
}

#[test]
fn ampersand_marks_pipeline_background() {
    let list = parse("sleep 1 &").unwrap().unwrap();
    assert_eq!(list.pipelines.len(), 1);
    assert!(list.pipelines[0].background);
    assert!(list.as_single_command().is_none());
}

#[test]
fn ampersand_separates_pipelines() {
    let list = parse("true & false").unwrap().unwrap();
    assert_eq!(list.pipelines.len(), 2);
    assert!(list.pipelines[0].background);
    assert!(!list.pipelines[1].background);
}

#[test]
fn leading_ampersand_is_null_command() {
    assert_eq!(parse("& true").unwrap_err(), ParseError::NullCommand);
}

#[test]
fn missing_redirect_target_is_error() {
    assert_eq!(
        parse("ls >").unwrap_err(),
        ParseError::MissingRedirectTarget
    );
    assert_eq!(
        parse("cat <").unwrap_err(),
        ParseError::MissingRedirectTarget
    );
    assert_eq!(
        parse("cat <<").unwrap_err(),
        ParseError::MissingRedirectTarget
    );
}

#[test]
fn fill_argv_reuses_string_capacity() {
    let mut env = ShellEnvironment::default();
    let mut argv = Vec::new();
    let mut stderr = Vec::new();
    fill_argv(
        &["one", "two"],
        &mut argv,
        &mut env,
        0,
        &mut std::io::empty(),
        &mut stderr,
    )
    .unwrap();
    assert_eq!(argv, ["one", "two"]);

    fill_argv(
        &["aaa", "bbb"],
        &mut argv,
        &mut env,
        0,
        &mut std::io::empty(),
        &mut stderr,
    )
    .unwrap();
    assert_eq!(argv, ["aaa", "bbb"]);
}

#[test]
fn fill_argv_expands_quotes() {
    let mut env = ShellEnvironment::default();
    let mut argv = Vec::new();
    let mut stderr = Vec::new();
    fill_argv(
        &[r#""hello world""#, r"a\|b"],
        &mut argv,
        &mut env,
        0,
        &mut std::io::empty(),
        &mut stderr,
    )
    .unwrap();
    assert_eq!(argv, ["hello world", "a|b"]);
}
