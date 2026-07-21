//! Tests for `setenv` / `unsetenv` together.

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
fn setenv_and_unsetenv() {
    let mut shell_env = empty_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        status_of(
            builtins::try_run(
                &["setenv".into(), "FOO".into(), "bar".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert_eq!(shell_env.get("FOO"), Some("bar"));
    assert_eq!(
        status_of(
            builtins::try_run(
                &["unsetenv".into(), "FOO".into()],
                &mut shell_env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap()
        ),
        0
    );
    assert!(!shell_env.contains("FOO"));
}
