//! Layered startup files (`.nexusenv`, `.nexusrc`, `.nexuslogin`, `NEXUS_NORCS`).

use nexus::env::ShellEnvironment;
use nexus::repl::{load_startup_chain, load_startup_env, RcLoad};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

static STARTUP_LOCK: Mutex<()> = Mutex::new(());

fn startup_lock() -> std::sync::MutexGuard<'static, ()> {
    STARTUP_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct DotdirGuard {
    prev: Option<std::ffi::OsString>,
}

impl Drop for DotdirGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(prev) => std::env::set_var("NEXUS_DOTDIR", prev),
            None => std::env::remove_var("NEXUS_DOTDIR"),
        }
    }
}

struct NorcsGuard {
    prev: Option<std::ffi::OsString>,
}

impl Drop for NorcsGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(prev) => std::env::set_var("NEXUS_NORCS", prev),
            None => std::env::remove_var("NEXUS_NORCS"),
        }
    }
}

fn temp_dotdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_dotdir_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn with_dotdir(dir: &PathBuf) -> DotdirGuard {
    let prev = std::env::var_os("NEXUS_DOTDIR");
    std::env::set_var("NEXUS_DOTDIR", dir);
    DotdirGuard { prev }
}

fn with_norcs() -> NorcsGuard {
    let prev = std::env::var_os("NEXUS_NORCS");
    std::env::set_var("NEXUS_NORCS", "1");
    NorcsGuard { prev }
}

fn write_startup(dir: &Path, name: &str, body: &str) {
    fs::write(dir.join(name), body).unwrap();
}

fn nexus_bin() -> PathBuf {
    env!("CARGO_BIN_EXE_nexus").into()
}

fn write_temp_script(name: &str, body: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("nexus_layers_{name}_{}.sh", std::process::id()));
    fs::write(&path, body).unwrap();
    path
}

#[test]
fn nexusenv_loads_on_script_path() {
    let _lock = startup_lock();
    let dotdir = temp_dotdir("script_env");
    write_startup(&dotdir, ".nexusenv", "set LAYER_ENV=from_nexusenv\n");
    let _dot = with_dotdir(&dotdir);
    let out = std::env::temp_dir().join(format!("nexus_layers_env_out_{}.txt", std::process::id()));
    let _ = fs::remove_file(&out);
    let script = write_temp_script(
        "check_env",
        &format!("printf '%s' \"$LAYER_ENV\" > {}\n", out.display()),
    );
    let status = Command::new(nexus_bin()).arg(&script).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(&out).unwrap(), "from_nexusenv");
    let _ = fs::remove_file(script);
    let _ = fs::remove_file(out);
    let _ = fs::remove_dir_all(dotdir);
}

#[test]
fn nexusrc_not_loaded_for_scripts() {
    let _lock = startup_lock();
    let dotdir = temp_dotdir("script_rc_skip");
    write_startup(&dotdir, ".nexusenv", ":\n");
    write_startup(&dotdir, ".nexusrc", "set LAYER_RC=should_not_load\n");
    let _dot = with_dotdir(&dotdir);
    let out = std::env::temp_dir().join(format!("nexus_layers_rc_out_{}.txt", std::process::id()));
    let _ = fs::remove_file(&out);
    let script = write_temp_script(
        "check_rc",
        &format!("printf '%s' \"${{LAYER_RC:-unset}}\" > {}\n", out.display()),
    );
    let status = Command::new(nexus_bin()).arg(&script).status().unwrap();
    assert!(status.success());
    assert_eq!(fs::read_to_string(&out).unwrap(), "unset");
    let _ = fs::remove_file(script);
    let _ = fs::remove_file(out);
    let _ = fs::remove_dir_all(dotdir);
}

#[test]
fn login_file_loads_only_when_login_true() {
    let _lock = startup_lock();
    let dotdir = temp_dotdir("login_layer");
    write_startup(&dotdir, ".nexusenv", ":\n");
    write_startup(&dotdir, ".nexuslogin", "set LAYER_LOGIN=yes\n");
    let _dot = with_dotdir(&dotdir);
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let env_only = load_startup_chain(false, true, &mut env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(env_only, RcLoad::Continue(0));
    assert_eq!(env.lookup("LAYER_LOGIN"), None);
    let loaded = load_startup_chain(true, true, &mut env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(loaded, RcLoad::Continue(0));
    assert_eq!(env.lookup("LAYER_LOGIN"), Some("yes"));
    let _ = fs::remove_dir_all(dotdir);
}

#[test]
fn nexus_norcs_skips_all_startup_files() {
    let _lock = startup_lock();
    let dotdir = temp_dotdir("norcs");
    write_startup(&dotdir, ".nexusenv", "set LAYER_NORCS=leaked\n");
    write_startup(&dotdir, ".nexusrc", "set LAYER_NORCS=leaked\n");
    let _dot = with_dotdir(&dotdir);
    let _norcs = with_norcs();
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        load_startup_env(&mut env, &mut stdout, &mut stderr).unwrap(),
        RcLoad::Skipped
    );
    assert_eq!(
        load_startup_chain(true, true, &mut env, &mut stdout, &mut stderr).unwrap(),
        RcLoad::Skipped
    );
    assert_eq!(env.lookup("LAYER_NORCS"), None);
    let _ = fs::remove_dir_all(dotdir);
}
