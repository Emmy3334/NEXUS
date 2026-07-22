//! Startup RC (`~/.nexusrc` / `NEXUSRC`) loading.

use nexus::env::ShellEnvironment;
use nexus::repl::{load_startup_rc, source_rc, RcLoad};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static NEXUSRC_LOCK: Mutex<()> = Mutex::new(());

fn temp_rc(name: &str, body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("nexus_rc_{name}_{}.nexusrc", std::process::id()));
    fs::write(&path, body).unwrap();
    path
}

struct NexusRcGuard(Option<std::ffi::OsString>);

impl Drop for NexusRcGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(prev) => std::env::set_var("NEXUSRC", prev),
            None => std::env::remove_var("NEXUSRC"),
        }
    }
}

fn with_nexusrc(path: &str) -> NexusRcGuard {
    let prev = std::env::var_os("NEXUSRC");
    std::env::set_var("NEXUSRC", path);
    NexusRcGuard(prev)
}

#[test]
fn source_rc_missing_is_quiet_noop() {
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let path = std::env::temp_dir().join("nexus_rc_missing_definitely_not_there");
    let _ = fs::remove_file(&path);
    let ended = source_rc(&path, &mut env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(ended, RcLoad::Skipped);
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn source_rc_applies_set_and_alias() {
    let path = temp_rc("apply", "set FOO=from_rc\nalias ll ls\n");
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let ended = source_rc(&path, &mut env, &mut stdout, &mut stderr).unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(ended, RcLoad::Continue(0));
    assert_eq!(env.lookup("FOO"), Some("from_rc"));
    assert_eq!(env.alias_get("ll"), Some("ls"));
}

#[test]
fn source_rc_exit_returns_code() {
    let path = temp_rc("exit", "exit 7\n");
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let ended = source_rc(&path, &mut env, &mut stdout, &mut stderr).unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(ended, RcLoad::Exit(7));
}

#[test]
fn source_rc_preserves_last_status() {
    let path = temp_rc("status", "false\n");
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let ended = source_rc(&path, &mut env, &mut stdout, &mut stderr).unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(ended, RcLoad::Continue(1));
}

#[test]
fn load_startup_rc_honors_nexusrc_override() {
    let _lock = NEXUSRC_LOCK.lock().unwrap();
    let path = temp_rc("override", "set BAR=via_nexusrc\n");
    let _guard = with_nexusrc(path.to_str().unwrap());
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let ended = load_startup_rc(&mut env, &mut stdout, &mut stderr).unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(ended, RcLoad::Continue(0));
    assert_eq!(env.lookup("BAR"), Some("via_nexusrc"));
}

#[test]
fn empty_nexusrc_disables_load() {
    let _lock = NEXUSRC_LOCK.lock().unwrap();
    let _guard = with_nexusrc("");
    let mut env = ShellEnvironment::default();
    env.alias_set("keep", "me");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let ended = load_startup_rc(&mut env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(ended, RcLoad::Skipped);
    assert_eq!(env.alias_get("keep"), Some("me"));
}
