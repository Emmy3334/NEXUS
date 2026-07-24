//! Command hash cache and PATH invalidation.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::pathfind;
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn with_hash_test<F: FnOnce()>(f: F) {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    pathfind::clear_hash();
    f();
}

fn temp_bin(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_hash_test_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let bin = dir.join("nexus_fake_cmd");
    fs::write(&bin, b"#!/bin/sh\necho ok\n").unwrap();
    let mut perms = fs::metadata(&bin).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&bin, perms).unwrap();
    dir
}

fn env_with_path(path: &str) -> ShellEnvironment {
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), path.into());
    ShellEnvironment::from_map(map)
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
}

#[test]
fn resolve_first_caches_executable() {
    with_hash_test(|| {
        let dir = temp_bin("cache");
        let path_var = dir.display().to_string();
        let first = pathfind::resolve_first("nexus_fake_cmd", &path_var).unwrap();
        let again = pathfind::resolve_first("nexus_fake_cmd", &path_var).unwrap();
        assert_eq!(first, again);

        fs::remove_file(dir.join("nexus_fake_cmd")).unwrap();
        assert!(pathfind::resolve_first("nexus_fake_cmd", &path_var).is_none());
        let _ = fs::remove_dir_all(dir);
    });
}

#[test]
fn path_change_invalidates_cache() {
    with_hash_test(|| {
        let dir_a = temp_bin("path_a");
        let dir_b = temp_bin("path_b");
        let path_a = dir_a.display().to_string();
        let path_b = dir_b.display().to_string();

        let hit_a = pathfind::resolve_first("nexus_fake_cmd", &path_a).unwrap();
        assert_eq!(hit_a, dir_a.join("nexus_fake_cmd"));

        let hit_b = pathfind::resolve_first("nexus_fake_cmd", &path_b).unwrap();
        assert_eq!(hit_b, dir_b.join("nexus_fake_cmd"));
        assert_ne!(hit_a, hit_b);

        let _ = fs::remove_dir_all(dir_a);
        let _ = fs::remove_dir_all(dir_b);
    });
}

#[test]
fn hash_lists_and_clears_via_builtin() {
    with_hash_test(|| {
        let dir = temp_bin("builtin");
        let path_var = dir.display().to_string();
        let mut env = env_with_path(&path_var);
        let _ = pathfind::resolve_first("nexus_fake_cmd", &path_var);

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = status_of(
            builtins::try_run(&["hash".into()], &mut env, 0, &mut stdout, &mut stderr).unwrap(),
        );
        assert_eq!(code, 0);
        let listed = String::from_utf8(stdout.clone()).unwrap();
        assert!(listed.contains("nexus_fake_cmd="));
        assert!(listed.contains(&dir.join("nexus_fake_cmd").display().to_string()));

        stdout.clear();
        let code = status_of(
            builtins::try_run(
                &["hash".into(), "-r".into()],
                &mut env,
                0,
                &mut stdout,
                &mut stderr,
            )
            .unwrap(),
        );
        assert_eq!(code, 0);

        stdout.clear();
        status_of(
            builtins::try_run(&["hash".into()], &mut env, 0, &mut stdout, &mut stderr).unwrap(),
        );
        assert!(stdout.is_empty());

        let _ = fs::remove_dir_all(dir);
    });
}

#[test]
fn list_commands_uses_dir_cache() {
    with_hash_test(|| {
        let dir = temp_bin("list");
        let path_var = dir.display().to_string();
        let names = pathfind::list_commands("nexus", &path_var);
        assert!(names.iter().any(|n| n == "nexus_fake_cmd"));
        let again = pathfind::list_commands("nexus", &path_var);
        assert_eq!(names, again);
        let _ = fs::remove_dir_all(dir);
    });
}
