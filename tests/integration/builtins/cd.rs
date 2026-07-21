//! Tests for the `cd` builtin (via [`nexus::builtins::try_run`]).

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use std::collections::BTreeMap;
use std::env as process_env;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

static CWD_TEST_LOCK: Mutex<()> = Mutex::new(());

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
    CWD_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
}

#[test]
fn cd_changes_directory() {
    let _cwd_guard = lock_cwd();
    let start = process_env::current_dir().unwrap();
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

    process_env::set_current_dir(&start).unwrap();
    let _ = fs::remove_dir_all(&scratch);
}
