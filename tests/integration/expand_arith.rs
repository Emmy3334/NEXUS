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
    let mut env = test_env();
    expand_word_for_exec(raw, &mut env, 0).map(|w| w.into_string())
}

fn expand_status(raw: &str, status: u8) -> Result<String, LexError> {
    let mut env = test_env();
    expand_word_for_exec(raw, &mut env, status).map(|w| w.into_string())
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
    let word = expand_word_for_exec("$(( $n * 2 + $? ))", &mut env, 3)
        .unwrap()
        .into_string();
    assert_eq!(word, "11");
}

#[test]
fn bare_names() {
    let mut env = test_env();
    env.set_local("x", "10");
    env.set_local("y", "3");
    let word = expand_word_for_exec("$((x+y*2))", &mut env, 0)
        .unwrap()
        .into_string();
    assert_eq!(word, "16");
    assert_eq!(expand("$((missing+7))").unwrap(), "7");
}

#[test]
fn nested_arith() {
    assert_eq!(expand("$((1+$((2*3))))").unwrap(), "7");
    let mut tokens = Vec::new();
    tokenize_into("$((1+$((2))))", &mut tokens).unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].lexeme("$((1+$((2))))"), "$((1+$((2))))");
}

#[test]
fn power_compare_logic_bit() {
    assert_eq!(expand("$((2**8))").unwrap(), "256");
    assert_eq!(expand("$((2**3**2))").unwrap(), "512"); // right-assoc: 2**(3**2)
    assert_eq!(expand("$((3>2))").unwrap(), "1");
    assert_eq!(expand("$((3==3 && 1))").unwrap(), "1");
    assert_eq!(expand("$((0||5))").unwrap(), "1");
    assert_eq!(expand("$((!0))").unwrap(), "1");
    assert_eq!(expand("$((5&3))").unwrap(), "1");
    assert_eq!(expand("$((1<<4))").unwrap(), "16");
    assert_eq!(expand("$((~0))").unwrap(), "-1");
}

#[test]
fn ternary() {
    assert_eq!(expand("$((1?4:5))").unwrap(), "4");
    assert_eq!(expand("$((0?4:5))").unwrap(), "5");
    assert_eq!(expand("$((2>1?10:20))").unwrap(), "10");
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
    assert_eq!(expand("$((2**-1))"), Err(LexError::Arithmetic));
}

#[test]
fn unset_var_is_zero() {
    assert_eq!(expand("$(( $missing + 5 ))").unwrap(), "5");
}

#[test]
fn assignment_sets_local_and_returns_value() {
    let mut env = test_env();
    let word = expand_word_for_exec("$((x=5))", &mut env, 0)
        .unwrap()
        .into_string();
    assert_eq!(word, "5");
    assert_eq!(env.lookup("x"), Some("5"));
}

#[test]
fn compound_assignment_and_chain() {
    let mut env = test_env();
    env.set_local("x", "3");
    assert_eq!(
        expand_word_for_exec("$((x+=2))", &mut env, 0)
            .unwrap()
            .into_string(),
        "5"
    );
    assert_eq!(env.lookup("x"), Some("5"));
    assert_eq!(
        expand_word_for_exec("$((a=b=4))", &mut env, 0)
            .unwrap()
            .into_string(),
        "4"
    );
    assert_eq!(env.lookup("a"), Some("4"));
    assert_eq!(env.lookup("b"), Some("4"));
}

#[test]
fn assignment_updates_exported_when_no_local() {
    let mut env = test_env();
    env.set("N", "1");
    assert_eq!(
        expand_word_for_exec("$((N+=1))", &mut env, 0)
            .unwrap()
            .into_string(),
        "2"
    );
    assert_eq!(env.get("N"), Some("2"));
    assert!(env.get_local("N").is_none());
}

#[test]
fn equality_is_not_assignment() {
    assert_eq!(expand("$((1==1))").unwrap(), "1");
    assert_eq!(expand("$((1=2))"), Err(LexError::Arithmetic));
}

#[test]
fn assignment_div_by_zero_errors() {
    assert_eq!(expand("$((x/=0))"), Err(LexError::Arithmetic));
}

#[test]
fn assignment_inside_cmdsubst_does_not_leak() {
    use nexus::exec::{execute_list, CommandResult};
    let mut env = test_env();
    let source = "printf '%s' $(echo $((leak=9)))";
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens).unwrap();
    let list = nexus::parse::parse_line(source, &tokens).unwrap().unwrap();
    let mut argv = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        &mut env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut Vec::new(),
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(result, CommandResult::Status(0));
    assert!(env.lookup("leak").is_none());
}

#[test]
fn prefix_increment_updates_and_returns_new() {
    let mut env = test_env();
    env.set_local("x", "1");
    assert_eq!(
        expand_word_for_exec("$((++x))", &mut env, 0)
            .unwrap()
            .into_string(),
        "2"
    );
    assert_eq!(env.lookup("x"), Some("2"));
}

#[test]
fn postfix_increment_returns_old_then_updates() {
    let mut env = test_env();
    env.set_local("x", "1");
    assert_eq!(
        expand_word_for_exec("$((x++))", &mut env, 0)
            .unwrap()
            .into_string(),
        "1"
    );
    assert_eq!(env.lookup("x"), Some("2"));
}

#[test]
fn prefix_decrement_and_unset_starts_at_zero() {
    let mut env = test_env();
    assert_eq!(
        expand_word_for_exec("$((++z))", &mut env, 0)
            .unwrap()
            .into_string(),
        "1"
    );
    assert_eq!(env.lookup("z"), Some("1"));
    assert_eq!(
        expand_word_for_exec("$((--z))", &mut env, 0)
            .unwrap()
            .into_string(),
        "0"
    );
    assert_eq!(env.lookup("z"), Some("0"));
}

#[test]
fn postfix_in_expression() {
    let mut env = test_env();
    env.set_local("x", "3");
    assert_eq!(
        expand_word_for_exec("$((x++ + 5))", &mut env, 0)
            .unwrap()
            .into_string(),
        "8"
    );
    assert_eq!(env.lookup("x"), Some("4"));
}

#[test]
fn inc_on_non_lvalue_errors() {
    assert_eq!(expand("$((1++))"), Err(LexError::Arithmetic));
}
