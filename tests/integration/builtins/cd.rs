//! Tests for the `cd` builtin (via [`nexus::builtins::try_run`]).

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use std::collections::BTreeMap;
use std::env as process_env;
use std::fs;
use std::path::Path;

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
fn cd_changes_directory() {
    let cwd = crate::cwd_lock::RestoreCwd::new();
    let scratch = process_env::temp_dir().join(format!("nexus-cd-test-{}", std::process::id()));
    let nested = scratch.join("nested");
    fs::create_dir_all(&nested).unwrap();

    let mut shell_env = empty_env();
    shell_env.set("HOME", scratch.to_string_lossy());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = status_of(
        builtins::try_run(
            &["cd".into(), nested.to_string_lossy().into_owned()],
            &mut shell_env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap(),
    );
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(
        process_env::current_dir().unwrap().canonicalize().unwrap(),
        nested.canonicalize().unwrap()
    );
    assert!(Path::new(shell_env.get("PWD").unwrap()).exists());
    assert_eq!(
        shell_env.get_local("cwd"),
        Some(shell_env.get("PWD").unwrap())
    );

    let code = status_of(
        builtins::try_run(
            &["cd".into(), "~/nested".into()],
            &mut shell_env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap(),
    );
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));

    let _ = cwd;
    let _ = fs::remove_dir_all(&scratch);
}
