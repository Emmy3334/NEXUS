//! Arithmetic expansion `$((…))`.

use nexus::env::ShellEnvironment;
use nexus::expand::expand_word_for_exec;
use nexus::lex::{tokenize_into, LexError, TokenKind};
use std::collections::BTreeMap;

fn test_env() -> ShellEnvironment {
    let mut map = BTreeMap::new();
    if let Ok(path) = std::env::var("PATH") {
        map.insert("PATH".into(), path);
    }
    ShellEnvironment::from_map(map)
}

fn expand(raw: &str) -> Result<String, LexError> {
    let env = test_env();
    expand_word_for_exec(raw, &env, 0).map(|w| w.into_string())
}

fn expand_status(raw: &str, status: u8) -> Result<String, LexError> {
    let env = test_env();
    expand_word_for_exec(raw, &env, status).map(|w| w.into_string())
}

#[test]
fn arith_is_single_lex_word() {
    let mut tokens = Vec::new();
    tokenize_into("echo $((1+2))", &mut tokens).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[1].kind, TokenKind::Word);
    assert_eq!(tokens[1].lexeme("echo $((1+2))"), "$((1+2))");
}

#[test]
fn basic_ops_and_parens() {
    assert_eq!(expand("$((1+2*3))").unwrap(), "7");
    assert_eq!(expand("$(( (1+2)*3 ))").unwrap(), "9");
    assert_eq!(expand("$((10/3))").unwrap(), "3");
    assert_eq!(expand("$((10%3))").unwrap(), "1");
    assert_eq!(expand("$((-4+1))").unwrap(), "-3");
}

#[test]
fn dollar_vars_inside() {
    let mut env = test_env();
    env.set_local("n", "4");
    let word = expand_word_for_exec("$(( $n * 2 + $? ))", &env, 3)
        .unwrap()
        .into_string();
    assert_eq!(word, "11");
}

#[test]
fn quoted_and_status() {
    assert_eq!(expand_status("\"$(( $? + 1 ))\"", 7).unwrap(), "8");
    assert_eq!(expand("$(($status+1))").unwrap(), "1");
}

#[test]
fn errors() {
    assert_eq!(expand("$((1/0))"), Err(LexError::Arithmetic));
    assert_eq!(expand("$((1+)"), Err(LexError::UnclosedArithmetic));
    assert_eq!(expand("$((1+))"), Err(LexError::Arithmetic));
}

#[test]
fn unset_var_is_zero() {
    assert_eq!(expand("$(( $missing + 5 ))").unwrap(), "5");
}
