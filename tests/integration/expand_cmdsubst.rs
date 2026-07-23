//! Command substitution via `$(…)`.

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::expand::expand_word_for_exec;
use nexus::lex::{tokenize_into, LexError, TokenKind};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn env_with(pairs: &[(&str, &str)]) -> ShellEnvironment {
    let mut map = BTreeMap::new();
    for &(k, v) in pairs {
        map.insert(k.into(), v.into());
    }
    if let Ok(path) = std::env::var("PATH") {
        map.entry("PATH".into()).or_insert(path);
    }
    ShellEnvironment::from_map(map)
}

fn temp_out(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_cmdsubst_{name}_{}", std::process::id()))
}

fn parse_list(source: &str) -> nexus::parse::CommandList<'_> {
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens).expect("lex ok");
    nexus::parse::parse_line(source, &tokens)
        .expect("parse ok")
        .expect("non-empty list")
}

fn run(source: &str, env: &mut ShellEnvironment, last_status: u8) -> (CommandResult, String) {
    let list = parse_list(source);
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        env,
        last_status,
        Vec::new(),
        &mut std::io::empty(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    (result, String::from_utf8(stderr).unwrap())
}

#[test]
fn cmdsubst_is_single_lex_word() {
    let src = "echo $(printf a b)";
    let mut tokens = Vec::new();
    tokenize_into(src, &mut tokens).unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[1].kind, TokenKind::Word);
    assert_eq!(tokens[1].lexeme(src), "$(printf a b)");
}

#[test]
fn arith_still_wins_over_cmdsubst_lex() {
    let src = "echo $((1+2))";
    let mut tokens = Vec::new();
    tokenize_into(src, &mut tokens).unwrap();
    assert_eq!(tokens[1].lexeme(src), "$((1+2))");
}

#[test]
fn unquoted_splits_on_whitespace() {
    let out = temp_out("split");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' $(printf 'a b c') > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "a\nb\nc\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn double_quoted_keeps_blanks() {
    let out = temp_out("dq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run(r#"printf '%s\n' "$(printf 'a b')" > $OUT"#, &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "a b\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn single_quoted_is_literal() {
    let out = temp_out("sq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' '$(echo hi)' > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "$(echo hi)\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn strips_trailing_newlines() {
    let out = temp_out("nl");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run("printf '%s' $(printf 'hi\\n\\n') > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "hi");
    let _ = fs::remove_file(&out);
}

#[test]
fn nested_cmdsubst() {
    let out = temp_out("nest");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("OUT", out_s)]);
    let (result, err) = run("printf '%s' $(printf $(printf hi)) > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert_eq!(fs::read_to_string(&out).unwrap(), "hi");
    let _ = fs::remove_file(&out);
}

#[test]
fn env_isolation() {
    let mut env = env_with(&[]);
    let (result, err) = run("$(set FOO bar); printf '%s' \"$FOO\"", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert!(env.lookup("FOO").is_none());
}

#[test]
fn unclosed_errors() {
    let mut tokens = Vec::new();
    assert_eq!(
        tokenize_into("echo $(hi", &mut tokens),
        Err(LexError::UnclosedCommandSubst)
    );
    assert_eq!(
        expand_word_for_exec("$(hi", &ShellEnvironment::default(), 0).unwrap_err(),
        LexError::UnclosedCommandSubst
    );
}

#[test]
fn in_redirect_path() {
    let path = temp_out("redir");
    let _ = fs::remove_file(&path);
    let path_s = path.to_str().expect("utf8 path");
    let mut env = env_with(&[]);
    let cmd = format!("printf 'ok\\n' > $(printf '{path_s}')");
    let (result, err) = run(&cmd, &mut env, 0);
    assert_eq!(result, CommandResult::Status(0), "{err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "ok\n");
    let _ = fs::remove_file(&path);
}
