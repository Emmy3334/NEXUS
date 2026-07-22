//! Tests for the `unsetenv` builtin.

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
fn unsetenv_rejects_wildcard() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(
        builtins::try_run(
            &["unsetenv".into(), "FOO*".into()],
            &mut shell_env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap(),
    );
    assert_eq!(code, 1);
    assert!(String::from_utf8(stderr).unwrap().contains("Wildcard"));
}
