//! Tests for `set` / `unset` shell-local builtins.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::expand::expand_word_for_exec;
use std::collections::BTreeMap;

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
}

#[test]
fn set_assigns_local_not_exported() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["set".into(), "FOO=bar".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert_eq!(shell_env.get_local("FOO"), Some("bar"));
    assert_eq!(shell_env.get("FOO"), None);
    assert_eq!(
        expand_word_for_exec("$FOO", &mut shell_env, 0)
            .unwrap()
            .as_str(),
        "bar"
    );
}

#[test]
fn set_name_equals_value_with_spaces() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["set".into(), "FOO".into(), "=".into(), "baz".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert_eq!(shell_env.get_local("FOO"), Some("baz"));
}

#[test]
fn set_name_only_sets_empty() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["set".into(), "EMPTY".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert_eq!(shell_env.get_local("EMPTY"), Some(""));
}

#[test]
fn bare_set_lists_locals() {
    let mut shell_env = empty_env();
    shell_env.set_local("A", "1");
    shell_env.set_local("B", "2");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(&["set".into()], &mut shell_env, 0, &mut stdout, &mut stderr)
                .unwrap()
        ),
        0
    );
    let out = String::from_utf8(stdout).unwrap();
    assert_eq!(out, "A=1\nB=2\n");
}

#[test]
fn unset_removes_local() {
    let mut shell_env = empty_env();
    shell_env.set_local("FOO", "bar");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["unset".into(), "FOO".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert_eq!(shell_env.get_local("FOO"), None);
    assert_eq!(
        expand_word_for_exec("$FOO", &mut shell_env, 0)
            .unwrap()
            .as_str(),
        ""
    );
}

#[test]
fn unset_rejects_wildcard() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["unset".into(), "F*".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        1
    );
    assert!(String::from_utf8(stderr).unwrap().contains("Wildcard"));
}

#[test]
fn set_rejects_invalid_name() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["set".into(), "1bad=x".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        1
    );
}

#[test]
fn local_not_inherited_by_env_builtin() {
    let mut shell_env = empty_env();
    shell_env.set_local("SECRET", "nope");
    shell_env.set("VISIBLE", "yes");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(&["env".into()], &mut shell_env, 0, &mut stdout, &mut stderr)
                .unwrap()
        ),
        0
    );
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("VISIBLE=yes"));
    assert!(!out.contains("SECRET"));
}
