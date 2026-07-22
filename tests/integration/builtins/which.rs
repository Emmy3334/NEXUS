//! Tests for `which` / `where`.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use std::collections::BTreeMap;

fn empty_env() -> ShellEnvironment {
    let mut env = ShellEnvironment::from_map(BTreeMap::new());
    env.set("PATH", "/bin:/usr/bin");
    env
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
}

#[test]
fn which_finds_builtin() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(
        builtins::try_run(
            &["which".into(), "cd".into()],
            &mut env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap(),
    );
    assert_eq!(code, 0);
    assert!(String::from_utf8(stdout)
        .unwrap()
        .contains("shell built-in"));
}

#[test]
fn which_missing_is_error() {
    let mut env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(
        builtins::try_run(
            &["which".into(), "nexus_no_such_cmd_zz".into()],
            &mut env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap(),
    );
    assert_eq!(code, 1);
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Command not found"));
}
