//! Tests for the `env` builtin.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
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
fn env_prints_sorted_assignments() {
    let mut shell_env = empty_env();
    shell_env.set("B", "2");
    shell_env.set("A", "1");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(
        builtins::try_run(&["env".into()], &mut shell_env, 0, &mut stdout, &mut stderr).unwrap(),
    );
    assert_eq!(code, 0);
    assert_eq!(String::from_utf8(stdout).unwrap(), "A=1\nB=2\n");
    assert!(stderr.is_empty());
}

#[test]
fn env_rejects_arguments() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(
        builtins::try_run(
            &["env".into(), "EXTRA".into()],
            &mut shell_env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap(),
    );
    assert_eq!(code, 1);
    assert!(String::from_utf8(stderr).unwrap().contains("Too many"));
}
