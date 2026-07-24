//! Integration tests for the `[[ … ]]` conditional builtin.

use nexus::repl;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn test_env() -> nexus::env::ShellEnvironment {
    let path = std::env::var("PATH").unwrap_or_default();
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), path);
    nexus::env::ShellEnvironment::from_map(map)
}

fn run(script: &str) -> (u8, String) {
    let mut env = test_env();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run_with_env(
        Cursor::new(script),
        &mut stdout,
        &mut stderr,
        false,
        false,
        &mut env,
    )
    .unwrap();
    (code, String::from_utf8(stderr).unwrap())
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_cond_{name}_{}", std::process::id()))
}

#[test]
fn file_tests() {
    let dir = scratch("files");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("f.txt");
    fs::write(&file, "x").unwrap();

    let (code, err) = run(&format!("[[ -f {} ]]\n", file.display()));
    assert_eq!(code, 0, "{err}");
    let (code, err) = run(&format!("[[ -d {} ]]\n", dir.display()));
    assert_eq!(code, 0, "{err}");
    let (code, err) = run(&format!("[[ -e {} ]]\n", file.display()));
    assert_eq!(code, 0, "{err}");
    let (code, _) = run("[[ -f /no/such/file ]]\n");
    assert_eq!(code, 1);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn string_and_integer_ops() {
    let (code, err) = run("[[ hello = hello ]]\n");
    assert_eq!(code, 0, "{err}");
    let (code, _) = run("[[ hello != world ]]\n");
    assert_eq!(code, 0);
    let (code, _) = run("[[ -z '' ]]\n");
    assert_eq!(code, 0);
    let (code, _) = run("[[ -n hi ]]\n");
    assert_eq!(code, 0);
    let (code, err) = run("[[ 3 -lt 10 ]]\n");
    assert_eq!(code, 0, "{err}");
    let (code, _) = run("[[ 3 -eq 4 ]]\n");
    assert_eq!(code, 1);
}

#[test]
fn chain_with_shell_and() {
    let (code, err) = run("[[ -n a ]] && [[ -z '' ]]\n");
    assert_eq!(code, 0, "{err}");
    let (code, _) = run("[[ -n a ]] && [[ -n '' ]]\n");
    assert_eq!(code, 1);
}

#[test]
fn logical_operators_work_inside_conditional() {
    let (code, err) = run("[[ -n a && -z '' ]]\n");
    assert_eq!(code, 0, "{err}");
    let (code, err) = run("[[ -z a || 4 -gt 2 ]]\n");
    assert_eq!(code, 0, "{err}");
}

#[test]
fn missing_close_is_error() {
    let (code, err) = run("[[ -n a\n");
    assert_eq!(code, 1);
    assert!(err.contains("[["), "{err}");
}
