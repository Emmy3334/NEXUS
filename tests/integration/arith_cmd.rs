//! Bash/ksh-style `((expr))` arithmetic command.

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::lex::tokenize_into;
use nexus::parse::parse_line;
use std::collections::BTreeMap;

fn test_env() -> ShellEnvironment {
    let mut map = BTreeMap::new();
    if let Ok(path) = std::env::var("PATH") {
        map.insert("PATH".into(), path);
    }
    ShellEnvironment::from_map(map)
}

fn run(source: &str, env: &mut ShellEnvironment) -> CommandResult {
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens).unwrap();
    let list = parse_line(source, &tokens).unwrap().unwrap();
    let mut argv = Vec::new();
    execute_list(
        &list,
        &mut argv,
        env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut Vec::new(),
        &mut Vec::new(),
    )
    .unwrap()
}

#[test]
fn arith_cmd_is_one_lex_word() {
    let mut tokens = Vec::new();
    tokenize_into("((x=1))", &mut tokens).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].lexeme("((x=1))"), "((x=1))");
}

#[test]
fn assignment_sets_var_without_running_result() {
    let mut env = test_env();
    assert_eq!(run("((x=1))", &mut env), CommandResult::Status(0));
    assert_eq!(env.lookup("x"), Some("1"));
}

#[test]
fn zero_result_is_status_one() {
    let mut env = test_env();
    assert_eq!(run("((0))", &mut env), CommandResult::Status(1));
    assert_eq!(run("((1-1))", &mut env), CommandResult::Status(1));
}

#[test]
fn nonzero_result_is_status_zero() {
    let mut env = test_env();
    assert_eq!(run("((2))", &mut env), CommandResult::Status(0));
}

#[test]
fn prefix_increment_works() {
    let mut env = test_env();
    env.set_local("n", "1");
    assert_eq!(run("((++n))", &mut env), CommandResult::Status(0));
    assert_eq!(env.lookup("n"), Some("2"));
}

#[test]
fn postfix_then_echo_via_expansion() {
    let mut env = test_env();
    assert_eq!(run("((x=1))", &mut env), CommandResult::Status(0));
    assert_eq!(run("((++x))", &mut env), CommandResult::Status(0));
    assert_eq!(env.lookup("x"), Some("2"));
    assert_eq!(run("((x++))", &mut env), CommandResult::Status(0));
    assert_eq!(env.lookup("x"), Some("3"));
}

#[test]
fn spaced_subshell_still_two_parens() {
    let mut tokens = Vec::new();
    tokenize_into("( (true) )", &mut tokens).unwrap();
    assert!(tokens.len() >= 2);
    assert_eq!(tokens[0].kind, nexus::lex::TokenKind::LParen);
}

#[test]
fn div_by_zero_is_status_one() {
    let mut env = test_env();
    let mut err = Vec::new();
    let mut tokens = Vec::new();
    tokenize_into("((1/0))", &mut tokens).unwrap();
    let list = parse_line("((1/0))", &tokens).unwrap().unwrap();
    let mut argv = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut Vec::new(),
        &mut err,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(1));
    assert!(String::from_utf8_lossy(&err).contains("Arithmetic"));
}

#[test]
fn comma_and_bitwise_assign_in_arith_cmd() {
    let mut env = test_env();
    assert_eq!(run("((x=15, x&=6, x))", &mut env), CommandResult::Status(0));
    assert_eq!(env.lookup("x"), Some("6"));
    assert_eq!(
        run("((y=1, y<<=2, y-4))", &mut env),
        CommandResult::Status(1)
    );
    assert_eq!(env.lookup("y"), Some("4"));
}
