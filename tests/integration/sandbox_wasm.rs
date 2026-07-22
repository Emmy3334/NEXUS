//! Wasm sandbox builtin and cache heal backend.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::exec::execute_external;
use nexus::heal::{attach_wasm_backend, ResolverChain};
use nexus::sandbox::{self, install_from_wat_into};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static WASM_CACHE_LOCK: Mutex<()> = Mutex::new(());

const EXIT42: &str = r#"
(module
  (import "wasi_snapshot_preview1" "proc_exit" (func $exit (param i32)))
  (memory (export "memory") 1)
  (func $_start (call $exit (i32.const 42)))
  (export "_start" (func $_start))
)
"#;

fn temp_cache(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_wasm_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

struct CacheGuard(Option<std::ffi::OsString>);

impl Drop for CacheGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(prev) => std::env::set_var("NEXUS_WASM_CACHE", prev),
            None => std::env::remove_var("NEXUS_WASM_CACHE"),
        }
    }
}

fn with_cache(dir: &PathBuf) -> CacheGuard {
    let prev = std::env::var_os("NEXUS_WASM_CACHE");
    std::env::set_var("NEXUS_WASM_CACHE", dir);
    CacheGuard(prev)
}

#[test]
fn sandbox_runs_wasm_path_with_exit_code() {
    let dir = temp_cache("path");
    let path = install_from_wat_into(&dir, "exit42", EXIT42).unwrap();
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let argv = vec!["sandbox".into(), path.to_string_lossy().into_owned()];
    let result = builtins::try_run(&argv, &mut env, 0, &mut stdout, &mut stderr)
        .unwrap()
        .unwrap();
    let _ = fs::remove_dir_all(dir);
    assert_eq!(result, BuiltinResult::Status(42));
}

#[test]
fn sandbox_runs_cached_module_by_name() {
    let _lock = WASM_CACHE_LOCK.lock().unwrap();
    let dir = temp_cache("named");
    let _guard = with_cache(&dir);
    install_from_wat_into(&dir, "demo", EXIT42).unwrap();
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let argv = vec!["sandbox".into(), "demo".into()];
    let result = builtins::try_run(&argv, &mut env, 0, &mut stdout, &mut stderr)
        .unwrap()
        .unwrap();
    let _ = fs::remove_dir_all(dir);
    assert_eq!(result, BuiltinResult::Status(42));
}

#[test]
fn sandbox_too_few_args() {
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let argv = vec!["sandbox".into()];
    let result = builtins::try_run(&argv, &mut env, 0, &mut stdout, &mut stderr)
        .unwrap()
        .unwrap();
    assert_eq!(result, BuiltinResult::Status(1));
    assert!(String::from_utf8(stderr).unwrap().contains("Too few"));
}

#[test]
fn wasm_heal_runs_cached_missing_command() {
    let _lock = WASM_CACHE_LOCK.lock().unwrap();
    let dir = temp_cache("heal");
    let _guard = with_cache(&dir);
    install_from_wat_into(&dir, "nexus_wasm_heal_tool", EXIT42).unwrap();
    let mut env = ShellEnvironment::default();
    attach_wasm_backend(&mut env);
    assert!(!env.healers.is_empty());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(
        &["nexus_wasm_heal_tool"],
        &mut env,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    let _ = fs::remove_dir_all(dir);
    assert_eq!(code, 42, "stderr={}", String::from_utf8_lossy(&stderr));
}

#[test]
fn resolve_named_misses_without_file() {
    let _lock = WASM_CACHE_LOCK.lock().unwrap();
    let dir = temp_cache("miss");
    let _guard = with_cache(&dir);
    assert!(sandbox::resolve_named("no_such_module").is_none());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn soft_host_isolates_home_and_marks_sandbox() {
    let mut env = ShellEnvironment::default();
    env.set("HOME", "/should/not/leak");
    env.set("SECRET_TOKEN", "leak-me");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let argv = vec![
        "sandbox".into(),
        "sh".into(),
        "-c".into(),
        "printf '%s|%s|%s' \"$NEXUS_SANDBOX\" \"$HOME\" \"$SECRET_TOKEN\"".into(),
    ];
    let result = builtins::try_run(&argv, &mut env, 0, &mut stdout, &mut stderr)
        .unwrap()
        .unwrap();
    assert_eq!(
        result,
        BuiltinResult::Status(0),
        "stderr={}",
        String::from_utf8_lossy(&stderr)
    );
    let out = String::from_utf8(stdout).unwrap();
    let parts: Vec<&str> = out.split('|').collect();
    assert_eq!(parts.len(), 3, "out={out:?}");
    assert_eq!(parts[0], "1");
    assert!(
        parts[1].contains("nexus_sandbox_"),
        "HOME should be temp workdir, got {}",
        parts[1]
    );
    assert_ne!(parts[1], "/should/not/leak");
    assert!(parts[2].is_empty(), "SECRET_TOKEN must not pass through");
}

#[test]
fn empty_chain_still_classic_when_no_wasm() {
    let mut env = ShellEnvironment::default();
    env.healers = ResolverChain::empty();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(
        &["nexus_definitely_missing_xyz"],
        &mut env,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(code, 127);
}
