//! Tests for `echo`, `true`, `false`, and `:`.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use std::collections::BTreeMap;

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

fn run(argv: &[&str]) -> (u8, String, String) {
    let mut env = empty_env();
    let argv: Vec<String> = argv.iter().map(|s| (*s).to_owned()).collect();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = builtins::try_run(&argv, &mut env, 0, &mut stdout, &mut stderr)
        .expect("io")
        .expect("recognized");
    let code = match result {
        BuiltinResult::Status(c) => c,
        other => panic!("expected Status, got {other:?}"),
    };
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

#[test]
fn echo_prints_args_with_newline() {
    let (code, out, err) = run(&["echo", "hello", "world"]);
    assert_eq!(code, 0);
    assert!(err.is_empty());
    assert_eq!(out, "hello world\n");
}

#[test]
fn echo_n_suppresses_newline() {
    let (code, out, _) = run(&["echo", "-n", "hi"]);
    assert_eq!(code, 0);
    assert_eq!(out, "hi");
}

#[test]
fn true_is_status_zero() {
    let (code, out, err) = run(&["true"]);
    assert_eq!(code, 0);
    assert!(out.is_empty() && err.is_empty());
}

#[test]
fn false_is_status_one() {
    let (code, _, _) = run(&["false"]);
    assert_eq!(code, 1);
}

#[test]
fn colon_is_status_zero() {
    let (code, _, _) = run(&[":"]);
    assert_eq!(code, 0);
}

#[test]
fn which_echo_is_builtin() {
    let (code, out, _) = run(&["which", "echo"]);
    assert_eq!(code, 0);
    assert!(out.contains("shell built-in"), "{out}");
}
