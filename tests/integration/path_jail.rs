//! PATH jail: relative / empty components are not searched.

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::harden;
use nexus::lex::tokenize_into;
use nexus::parse::parse_line;
use nexus::pathfind;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn test_env() -> ShellEnvironment {
    let mut map = BTreeMap::new();
    if let Ok(path) = std::env::var("PATH") {
        map.insert("PATH".into(), path);
    }
    ShellEnvironment::from_map(map)
}

fn run(source: &str, env: &mut ShellEnvironment) -> CommandResult {
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens).unwrap();
    let list = parse_line(source, &tokens).unwrap().unwrap();
    let mut argv = Vec::new();
    execute_list(
        &list,
        &mut argv,
        env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut Vec::new(),
        &mut Vec::new(),
    )
    .unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_path_jail_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn sanitize_drops_relative_and_empty() {
    assert_eq!(
        pathfind::sanitize_path(".:/tmp/evil::/usr/bin"),
        "/tmp/evil:/usr/bin"
    );
    assert!(!pathfind::sanitize_path("relative/bin").contains("relative"));
}

#[test]
fn relative_path_component_not_used_for_bare_name() {
    let dir = temp_dir("evil");
    let decoy = dir.join("true");
    fs::write(&decoy, b"#!/bin/sh\necho DECOY\n").unwrap();
    let mut perms = fs::metadata(&decoy).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&decoy, perms).unwrap();

    let mut env = test_env();
    // Prefer decoy dir first; jail should skip the relative "." entry only —
    // use a relative component that points at the decoy.
    let rel = dir.strip_prefix("/").unwrap_or(dir.as_path());
    let jailed_path = format!("{}:/bin:/usr/bin", rel.display());
    env.set("PATH", &jailed_path);

    // Relative component is dropped; system `true` should still run if present.
    let path = harden::effective_path(&env);
    assert!(!path.split(':').any(|c| !c.starts_with('/')), "{path}");
    assert!(pathfind::resolve_first("true", &path).is_some());

    let result = run("true", &mut env);
    assert_eq!(result, CommandResult::Status(0));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn absolute_path_still_runs() {
    let mut env = test_env();
    let result = run("/bin/echo ok", &mut env);
    assert_eq!(result, CommandResult::Status(0));
}

#[test]
fn path_jail_can_be_disabled() {
    let mut env = test_env();
    env.set_local("path_jail", "0");
    assert!(!harden::path_jail_enabled(&env));
    let raw = ".:/usr/bin";
    env.set("PATH", raw);
    assert_eq!(harden::effective_path(&env), raw);
}
