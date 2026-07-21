//! Tests for the `exit` builtin.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use std::collections::BTreeMap;

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

#[test]
fn exit_with_code() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = builtins::try_run(
        &["exit".into(), "42".into()],
        &mut shell_env,
        0,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, Some(BuiltinResult::Exit(42)));
}

#[test]
fn exit_without_arg_uses_last_status() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = builtins::try_run(
        &["exit".into()],
        &mut shell_env,
        7,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(result, Some(BuiltinResult::Exit(7)));
}
