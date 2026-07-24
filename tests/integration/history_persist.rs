//! Session history auto-persist and RC isolation.

use nexus::env::ShellEnvironment;
use nexus::history::History;
use nexus::repl::{load_session_history, save_session_history, source_rc, RcLoad};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_hist_persist_{name}_{}", std::process::id()))
}

fn temp_rc(name: &str, body: &str) -> PathBuf {
    let path = scratch(name).with_extension("nexusrc");
    fs::write(&path, body).unwrap();
    path
}

#[test]
fn source_rc_does_not_pollute_history() {
    let path = temp_rc("nohist", "alias ll ls\nset FOO=1\n");
    let mut env = ShellEnvironment::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let ended = source_rc(&path, &mut env, &mut stdout, &mut stderr).unwrap();
    let _ = fs::remove_file(&path);
    assert_eq!(ended, RcLoad::Continue(0));
    assert!(env.history.is_empty(), "RC lines must not enter history");
    assert_eq!(env.alias_get("ll"), Some("ls"));
}

#[test]
fn load_session_history_missing_is_quiet() {
    let dir = scratch("missing");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("nope");
    let mut env = ShellEnvironment::default();
    env.set_local("histfile", file.to_string_lossy().as_ref());
    let mut stderr = Vec::new();
    load_session_history(&mut env, &mut stderr).unwrap();
    assert!(env.history.is_empty());
    assert!(
        stderr.is_empty(),
        "stderr={}",
        String::from_utf8_lossy(&stderr)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn session_history_round_trips_via_histfile() {
    let dir = scratch("round");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("hist");
    let mut env = ShellEnvironment::default();
    env.set_local("histfile", file.to_string_lossy().as_ref());
    env.history
        .push_at("echo one", UNIX_EPOCH + Duration::from_secs(1));
    env.history
        .push_at("echo two", UNIX_EPOCH + Duration::from_secs(2));
    let mut stderr = Vec::new();
    save_session_history(&env, &mut stderr).unwrap();
    assert!(
        stderr.is_empty(),
        "stderr={}",
        String::from_utf8_lossy(&stderr)
    );

    let mut loaded = ShellEnvironment::default();
    loaded.set_local("histfile", file.to_string_lossy().as_ref());
    load_session_history(&mut loaded, &mut stderr).unwrap();
    let lines: Vec<_> = loaded
        .history
        .iter()
        .map(|(_, e)| e.line.as_str())
        .collect();
    assert_eq!(lines, ["echo one", "echo two"]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn save_session_history_skips_empty_missing_file() {
    let dir = scratch("skip");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("never_created");
    let env = {
        let mut env = ShellEnvironment::default();
        env.set_local("histfile", file.to_string_lossy().as_ref());
        env
    };
    let mut stderr = Vec::new();
    save_session_history(&env, &mut stderr).unwrap();
    assert!(!file.exists(), "empty session must not create histfile");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn histfile_path_prefers_local_over_home() {
    let path = nexus::history::histfile_path(Some("/tmp/custom.hist"), Some("/home/me"));
    assert_eq!(path, PathBuf::from("/tmp/custom.hist"));
    let path = nexus::history::histfile_path(None, Some("/home/me"));
    assert_eq!(path, PathBuf::from("/home/me/.nexus_history"));
}

#[test]
fn load_appends_without_clearing_existing() {
    let dir = scratch("append");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("hist");
    let mut foreign = History::default();
    foreign.push_at("fromfile", UNIX_EPOCH + Duration::from_secs(5));
    foreign.save_to(&file).unwrap();

    let mut env = ShellEnvironment::default();
    env.set_local("histfile", file.to_string_lossy().as_ref());
    env.history.push("already");
    let mut stderr = Vec::new();
    load_session_history(&mut env, &mut stderr).unwrap();
    let lines: Vec<_> = env.history.iter().map(|(_, e)| e.line.as_str()).collect();
    assert_eq!(lines, ["already", "fromfile"]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn histsize_caps_retained_events() {
    let mut h = History::default();
    for i in 0..5 {
        h.push_limited(format!("line{i}"), 3);
    }
    let lines: Vec<_> = h.iter().map(|(_, e)| e.line.as_str()).collect();
    assert_eq!(lines, ["line2", "line3", "line4"]);
}

#[test]
fn histsize_zero_clears() {
    let mut h = History::default();
    h.push("keep");
    h.push_limited("gone", 0);
    assert!(h.is_empty());
}
