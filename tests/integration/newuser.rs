//! First-run `nexus-newuser-install` wizard (zsh-newuser-install analogue).

use nexus::env::ShellEnvironment;
use nexus::repl::{
    newuser_should_offer, write_configured_rc, write_minimal_rc, NewuserConfig, NewuserKeymap,
    NewuserPromptStyle,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

static LOCK: Mutex<()> = Mutex::new(());

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

struct EnvGuard {
    key: &'static str,
    prev: Option<std::ffi::OsString>,
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.prev.take() {
            Some(prev) => std::env::set_var(self.key, prev),
            None => std::env::remove_var(self.key),
        }
    }
}

fn temp_dotdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_newuser_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn with_dotdir(dir: &PathBuf) -> DotdirGuard {
    let prev = std::env::var_os("NEXUS_DOTDIR");
    std::env::set_var("NEXUS_DOTDIR", dir);
    DotdirGuard { prev }
}

fn set_env(key: &'static str, val: &str) -> EnvGuard {
    let prev = std::env::var_os(key);
    std::env::set_var(key, val);
    EnvGuard { key, prev }
}

#[test]
fn should_offer_on_empty_dotdir() {
    let _lock = LOCK.lock().unwrap();
    let dir = temp_dotdir("empty");
    let _dot = with_dotdir(&dir);
    std::env::remove_var("NEXUS_NONEWUSER");
    std::env::remove_var("NEXUSRC");
    std::env::remove_var("NEXUS_NEWUSER");
    let env = ShellEnvironment::default();
    assert!(newuser_should_offer(&env));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn should_not_offer_when_rc_exists() {
    let _lock = LOCK.lock().unwrap();
    let dir = temp_dotdir("has_rc");
    fs::write(dir.join(".nexusrc"), "# hi\n").unwrap();
    let _dot = with_dotdir(&dir);
    std::env::remove_var("NEXUS_NEWUSER");
    let env = ShellEnvironment::default();
    assert!(!newuser_should_offer(&env));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn should_not_offer_when_disabled() {
    let _lock = LOCK.lock().unwrap();
    let dir = temp_dotdir("disabled");
    let _dot = with_dotdir(&dir);
    let _g = set_env("NEXUS_NONEWUSER", "1");
    let env = ShellEnvironment::default();
    assert!(!newuser_should_offer(&env));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn write_minimal_and_configured_rc() {
    let dir = temp_dotdir("write");
    let minimal = dir.join("minimal.nexusrc");
    write_minimal_rc(&minimal).unwrap();
    let body = fs::read_to_string(&minimal).unwrap();
    assert!(body.contains("nexus-newuser-install"));

    let configured = dir.join("configured.nexusrc");
    let cfg = NewuserConfig {
        histsize: 5000,
        prompt_style: NewuserPromptStyle::Powerlevel10k,
        keymap: NewuserKeymap::Vi,
        oh_my_nexus: true,
    };
    write_configured_rc(&configured, &cfg).unwrap();
    let body = fs::read_to_string(&configured).unwrap();
    assert!(body.contains("set histsize = 5000"));
    assert!(body.contains("NEXUS_PROMPT_STYLE powerlevel10k"));
    assert!(body.contains("bindkey -v"));
    assert!(body.contains("oh-my-nexus.nexus"));
    let _ = fs::remove_dir_all(dir);
}
