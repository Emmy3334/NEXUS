//! Brace `{a,b}` and recursive `**` pathname expansion.

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::expand::{expand_word_fields_into, expand_word_for_exec};
use nexus::glob::expand_globs;
use nexus::lex::LexError;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn fields(raw: &str) -> Vec<String> {
    let env = ShellEnvironment::default();
    let mut out = Vec::new();
    let mut deny = |_: &str| Err(LexError::CommandSubstitution);
    expand_word_fields_into(raw, &env, 0, &mut out, &mut deny).unwrap();
    out.into_iter().map(|w| w.into_string()).collect()
}

fn test_env() -> ShellEnvironment {
    let path = std::env::var("PATH").unwrap_or_default();
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), path);
    ShellEnvironment::from_map(map)
}

fn parse_list(source: &str) -> nexus::parse::CommandList<'_> {
    let mut tokens = Vec::new();
    nexus::lex::tokenize_into(source, &mut tokens).expect("lex ok");
    nexus::parse::parse_line(source, &tokens)
        .expect("parse ok")
        .expect("non-empty list")
}

fn run(source: &str, env: &mut ShellEnvironment) -> (CommandResult, String) {
    let list = parse_list(source);
    let mut argv = Vec::new();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    (result, String::from_utf8(stderr).unwrap())
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_brace_glob_{name}_{}", std::process::id()))
}

#[test]
fn brace_expands_comma_list() {
    assert_eq!(fields("a{b,c}d"), vec!["abd".to_owned(), "acd".to_owned()]);
    assert_eq!(
        fields("{one,two}"),
        vec!["one".to_owned(), "two".to_owned()]
    );
}

#[test]
fn brace_nested_and_empty_alt() {
    assert_eq!(
        fields("x{a,b{c,d}}"),
        vec!["xa".to_owned(), "xbc".to_owned(), "xbd".to_owned()]
    );
    assert_eq!(fields("{a,}"), vec!["a".to_owned(), "".to_owned()]);
}

#[test]
fn brace_without_comma_stays_literal() {
    assert_eq!(fields("{abc}"), vec!["{abc}".to_owned()]);
}

#[test]
fn quoted_brace_stays_literal() {
    assert_eq!(fields("'{a,b}'"), vec!["{a,b}".to_owned()]);
    assert_eq!(fields("\"{a,b}\""), vec!["{a,b}".to_owned()]);
}

#[test]
fn brace_then_dollar() {
    let mut env = ShellEnvironment::default();
    env.set_local("X", "z");
    let mut out = Vec::new();
    let mut deny = |_: &str| Err(LexError::CommandSubstitution);
    expand_word_fields_into("pre{$X,y}", &env, 0, &mut out, &mut deny).unwrap();
    let got: Vec<_> = out.into_iter().map(|w| w.into_string()).collect();
    assert_eq!(got, vec!["prez".to_owned(), "prey".to_owned()]);
}

#[test]
fn exec_printf_brace_list() {
    let path = scratch("brace_out");
    let _ = fs::remove_file(&path);
    let mut env = test_env();
    let cmd = format!("printf '%s\\n' a{{b,c}} > {}", path.display());
    let (result, err) = run(&cmd, &mut env);
    assert_eq!(result, CommandResult::Status(0), "err={err}");
    assert_eq!(fs::read_to_string(&path).unwrap(), "ab\nac\n");
    let _ = fs::remove_file(&path);
}

#[test]
fn globstar_finds_nested_file() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("star");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("a/b")).unwrap();
    fs::write(dir.join("a/b/target.txt"), "").unwrap();
    fs::write(dir.join("skip.txt"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let word = expand_word_for_exec("**/target.txt", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(expand_globs(&word), vec!["a/b/target.txt".to_owned()]);

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn globstar_trailing_lists_descendants() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("trail");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("d")).unwrap();
    fs::write(dir.join("d/f"), "").unwrap();
    fs::write(dir.join("top"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let word = expand_word_for_exec("**", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(
        expand_globs(&word),
        vec!["d".to_owned(), "d/f".to_owned(), "top".to_owned()]
    );

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn brace_with_glob() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch("braceglob");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join("b.txt"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let mut env = test_env();
    let (result, err) = run("printf '%s\\n' {a,b}.txt", &mut env);
    assert_eq!(result, CommandResult::Status(0), "err={err}");

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}
