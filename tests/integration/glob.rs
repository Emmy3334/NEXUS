//! Integration tests for pathname expansion (`*`, `?`, `[…]`).

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::expand::expand_word_for_exec;
use nexus::glob::{expand_globs, expand_globs_one};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
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

fn scratch_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_glob_{name}_{}", std::process::id()))
}

#[test]
fn star_expands_sorted_non_hidden() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch_dir("star");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("b.txt"), "").unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join(".hidden"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let word = expand_word_for_exec("*", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(
        expand_globs(&word),
        vec!["a.txt".to_owned(), "b.txt".to_owned()]
    );

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn question_matches_one_char() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch_dir("q");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("ab"), "").unwrap();
    fs::write(dir.join("a"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let word = expand_word_for_exec("?", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(expand_globs(&word), vec!["a".to_owned()]);

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bracket_class_and_range() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch_dir("bracket");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a"), "").unwrap();
    fs::write(dir.join("b"), "").unwrap();
    fs::write(dir.join("c"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let word = expand_word_for_exec("[ab]", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(expand_globs(&word), vec!["a".to_owned(), "b".to_owned()]);

    let word = expand_word_for_exec("[a-c]", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(
        expand_globs(&word),
        vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]
    );

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn quoted_glob_stays_literal() {
    let word = expand_word_for_exec("'*'", &ShellEnvironment::default(), 0).unwrap();
    assert!(!word.has_active_glob());
    assert_eq!(expand_globs(&word), vec!["*".to_owned()]);

    let word = expand_word_for_exec("\"*\"", &ShellEnvironment::default(), 0).unwrap();
    assert!(!word.has_active_glob());
    assert_eq!(expand_globs(&word), vec!["*".to_owned()]);
}

#[test]
fn no_match_keeps_literal() {
    let word =
        expand_word_for_exec("nexus_no_such_glob_zzz*", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(
        expand_globs(&word),
        vec!["nexus_no_such_glob_zzz*".to_owned()]
    );
}

#[test]
fn exec_printf_globs_to_file() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch_dir("exec");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("one.dat"), "").unwrap();
    fs::write(dir.join("two.dat"), "").unwrap();
    let out = dir.join("out.txt");
    std::env::set_current_dir(&dir).unwrap();

    let mut env = test_env();
    let (result, err) = run("printf '%s\\n' *.dat > out.txt", &mut env);
    assert_eq!(result, CommandResult::Status(0), "err={err}");
    assert!(err.is_empty());
    let contents = fs::read_to_string(&out).unwrap();
    assert_eq!(contents, "one.dat\ntwo.dat\n");

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ambiguous_redirect_is_error() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch_dir("ambig");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join("b.txt"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let mut env = test_env();
    let (result, err) = run("printf x > *.txt", &mut env);
    assert_eq!(result, CommandResult::Status(1));
    assert!(err.contains("Ambiguous redirect"));

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn redirect_single_glob_ok() {
    let _guard = crate::cwd_lock::lock();
    let start = std::env::current_dir().unwrap();
    let dir = scratch_dir("one");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("only.out"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let word = expand_word_for_exec("*.out", &ShellEnvironment::default(), 0).unwrap();
    assert_eq!(expand_globs_one(&word).unwrap(), "only.out");

    let mut env = test_env();
    let (result, err) = run("printf 'hi\\n' > *.out", &mut env);
    assert_eq!(result, CommandResult::Status(0), "err={err}");
    assert_eq!(fs::read_to_string(dir.join("only.out")).unwrap(), "hi\n");

    std::env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&dir);
}
