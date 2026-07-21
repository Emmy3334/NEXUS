//! Tests for external command spawn.

use super::common::test_env;
use nexus::env::ShellEnvironment;
use nexus::exec::{execute_command, execute_external, CommandResult};
use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn true_exits_zero() {
    let env = test_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(&["true"], &env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
}

#[test]
fn false_exits_one() {
    let env = test_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(&["false"], &env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(code, 1);
    assert!(stderr.is_empty());
}

#[test]
fn missing_command_is_127_with_message() {
    let env = test_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(
        &["nexus_no_such_command_42"],
        &env,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(code, 127);
    let message = String::from_utf8(stderr).unwrap();
    assert!(message.contains("Command not found"));
    assert!(message.contains("nexus_no_such_command_42"));
}

#[test]
fn absolute_true_path() {
    let true_path = ["/usr/bin/true", "/bin/true"]
        .into_iter()
        .find(|path| Path::new(path).exists())
        .expect("system true binary");

    let env = test_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(&[true_path], &env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
}

#[test]
fn exit_builtin_stops_shell() {
    let mut env = test_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_command(
        &["exit".into(), "3".into()],
        &mut env,
        0,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, CommandResult::Exit(3));
}

#[test]
fn external_inherits_shell_path_only_env() {
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), "/nonexistent".into());
    let env = ShellEnvironment::from_map(map);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(&["true"], &env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(code, 127);
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Command not found"));
}
