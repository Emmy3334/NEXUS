//! Integration tests for `$` / `$?` word expansion at exec time.

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::expand::expand_word_for_exec;
use nexus::lex::LexError;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn env_with(pairs: &[(&str, &str)]) -> ShellEnvironment {
    let mut map = BTreeMap::new();
    for &(k, v) in pairs {
        map.insert(k.into(), v.into());
    }
    // Keep PATH so external helpers (`printf`, `cat`, …) resolve.
    if let Ok(path) = std::env::var("PATH") {
        map.entry("PATH".into()).or_insert(path);
    }
    ShellEnvironment::from_map(map)
}

fn parse_list(source: &str) -> nexus::parse::CommandList<'_> {
    let mut tokens = Vec::new();
    nexus::lex::tokenize_into(source, &mut tokens).expect("lex ok");
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
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    (result, String::from_utf8(stderr).unwrap())
}

fn temp_out(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_expand_{name}_{}", std::process::id()))
}

#[test]
fn expand_home_plain_and_braced() {
    let env = env_with(&[("HOME", "/tmp/home")]);
    assert_eq!(expand_word_for_exec("$HOME", &env, 0).unwrap(), "/tmp/home");
    assert_eq!(
        expand_word_for_exec("${HOME}", &env, 0).unwrap(),
        "/tmp/home"
    );
}

#[test]
fn expand_status_question() {
    let env = ShellEnvironment::default();
    assert_eq!(expand_word_for_exec("$?", &env, 42).unwrap(), "42");
    assert_eq!(expand_word_for_exec("x$?y", &env, 7).unwrap(), "x7y");
}

#[test]
fn expand_status_name_like_question() {
    let env = ShellEnvironment::default();
    assert_eq!(expand_word_for_exec("$status", &env, 3).unwrap(), "3");
    assert_eq!(expand_word_for_exec("${status}", &env, 11).unwrap(), "11");
}

#[test]
fn expand_cwd_local() {
    let mut env = ShellEnvironment::default();
    env.set_local("cwd", "/tmp/nexus_cwd");
    assert_eq!(
        expand_word_for_exec("$cwd", &env, 0).unwrap(),
        "/tmp/nexus_cwd"
    );
}

#[test]
fn no_expand_inside_single_quotes() {
    let env = env_with(&[("HOME", "/tmp/home")]);
    assert_eq!(expand_word_for_exec("'$HOME'", &env, 0).unwrap(), "$HOME");
    assert_eq!(expand_word_for_exec("'$?'", &env, 3).unwrap(), "$?");
}

#[test]
fn expand_inside_double_quotes() {
    let env = env_with(&[("HOME", "/tmp/home")]);
    assert_eq!(
        expand_word_for_exec("\"$HOME\"", &env, 0).unwrap(),
        "/tmp/home"
    );
    assert_eq!(
        expand_word_for_exec("\"status=$?\"", &env, 9).unwrap(),
        "status=9"
    );
}

#[test]
fn unknown_var_expands_to_empty() {
    let env = ShellEnvironment::default();
    assert_eq!(expand_word_for_exec("$NEXUS_NO_SUCH", &env, 0).unwrap(), "");
    assert_eq!(
        expand_word_for_exec("${NEXUS_NO_SUCH}", &env, 0).unwrap(),
        ""
    );
}

#[test]
fn escaped_dollar_stays_literal() {
    let env = env_with(&[("HOME", "/tmp")]);
    assert_eq!(expand_word_for_exec(r"\$HOME", &env, 0).unwrap(), "$HOME");
    assert_eq!(
        expand_word_for_exec(r#""\$HOME""#, &env, 0).unwrap(),
        "$HOME"
    );
}

#[test]
fn unclosed_quote_still_errors() {
    let env = ShellEnvironment::default();
    assert_eq!(
        expand_word_for_exec("\"$HOME", &env, 0).unwrap_err(),
        LexError::UnclosedQuote
    );
}

#[test]
fn exec_printf_home_and_status_to_file() {
    let out = temp_out("home_status");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("HOME", "/tmp/nexus_home"), ("OUT", out_s)]);

    let (result, err) = run("printf '%s\\n' $HOME > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "/tmp/nexus_home\n");

    let (result, err) = run("false ; printf '%s\\n' $? > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "1\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn exec_no_expand_in_single_quotes() {
    let out = temp_out("sq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("HOME", "/tmp/home"), ("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' '$HOME' > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "$HOME\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn exec_expand_in_double_quotes() {
    let out = temp_out("dq");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("HOME", "/tmp/home"), ("OUT", out_s)]);
    let (result, err) = run(r#"printf '%s\n' "$HOME" > $OUT"#, &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "/tmp/home\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn redirect_path_expands_var() {
    let path = temp_out("redir");
    let _ = fs::remove_file(&path);
    let mut env = env_with(&[("OUTFILE", path.to_str().expect("utf8 path"))]);
    let (result, err) = run("printf 'ok\\n' > $OUTFILE", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "ok\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn pipe_argv_expands_var() {
    let out = temp_out("pipe");
    let _ = fs::remove_file(&out);
    let out_s = out.to_str().expect("utf8 path");
    let mut env = env_with(&[("MSG", "piped"), ("OUT", out_s)]);
    let (result, err) = run("printf '%s\\n' $MSG | cat > $OUT", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&out).unwrap(), "piped\n");
    let _ = fs::remove_file(&out);
}

#[test]
fn local_shadows_exported_in_expansion() {
    let mut env = env_with(&[("FOO", "exported")]);
    env.set_local("FOO", "local");
    assert_eq!(expand_word_for_exec("$FOO", &env, 0).unwrap(), "local");
}

#[test]
fn redirect_path_uses_braced_form() {
    let path = temp_out("braced");
    let _ = fs::remove_file(&path);
    let mut env = env_with(&[("OUTFILE", path.to_str().expect("utf8 path"))]);
    let (result, err) = run("printf 'x\\n' > ${OUTFILE}", &mut env, 0);
    assert_eq!(result, CommandResult::Status(0));
    assert!(err.is_empty());
    assert_eq!(fs::read_to_string(&path).unwrap(), "x\n");
    let _ = fs::remove_file(&path);
}
