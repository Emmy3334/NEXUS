//! Builtin tests for `jobs` / `fg` / `bg`.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use std::collections::BTreeMap;

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

#[test]
fn jobs_with_no_jobs_prints_nothing() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result =
        builtins::try_run(&["jobs".into()], &mut env, 0, &mut stdout, &mut stderr).unwrap();
    assert_eq!(result, Some(BuiltinResult::Status(0)));
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn jobs_l_with_no_jobs_prints_nothing() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = builtins::try_run(
        &["jobs".into(), "-l".into()],
        &mut env,
        0,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, Some(BuiltinResult::Status(0)));
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn jobs_unknown_flag_errors() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = builtins::try_run(
        &["jobs".into(), "-x".into()],
        &mut env,
        0,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, Some(BuiltinResult::Status(1)));
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Too many arguments"));
}

#[test]
fn bg_with_no_jobs_errors() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = builtins::try_run(&["bg".into()], &mut env, 0, &mut stdout, &mut stderr).unwrap();
    assert_eq!(result, Some(BuiltinResult::Status(1)));
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("No current job"));
}

#[test]
fn disown_with_no_jobs_errors() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result =
        builtins::try_run(&["disown".into()], &mut env, 0, &mut stdout, &mut stderr).unwrap();
    assert_eq!(result, Some(BuiltinResult::Status(1)));
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("No current job"));
}
