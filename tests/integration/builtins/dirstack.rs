//! Tests for `pushd` / `popd` / `dirs` including tcsh flags.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::ShellEnvironment;
use nexus::repl;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_dirs_{name}_{}", std::process::id()))
}

fn status_of(result: Option<BuiltinResult>) -> u8 {
    match result {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("expected Status, got {other:?}"),
    }
}

#[test]
fn pushd_popd_round_trip() {
    let start = std::env::current_dir().unwrap();
    let dir = scratch("stack");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let script = format!("pushd {}\ndirs\npopd\ndirs\n", dir.display());
    let mut stdin = Cursor::new(script);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    let cwd_after = std::env::current_dir().unwrap();
    let _ = std::env::set_current_dir(&start);
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(code, 0, "err={}", String::from_utf8_lossy(&stderr));
    assert_eq!(cwd_after, start);
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains(dir.file_name().unwrap().to_str().unwrap()));
}

#[test]
fn dirs_v_prints_indexed_lines() {
    let start = std::env::current_dir().unwrap();
    let a = scratch("va");
    let b = scratch("vb");
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    let script = format!("pushd {}\npushd {}\ndirs -v\n", a.display(), b.display());
    let mut stdin = Cursor::new(script);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    let _ = std::env::set_current_dir(&start);
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    assert_eq!(code, 0, "err={}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("0\t"), "out={out}");
    assert!(out.contains("1\t"), "out={out}");
}

#[test]
fn dirs_c_clears_to_cwd_only() {
    let start = std::env::current_dir().unwrap();
    let dir = scratch("clear");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let script = format!("pushd {}\ndirs -c\ndirs -v\n", dir.display());
    let mut stdin = Cursor::new(script);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    let _ = std::env::set_current_dir(&start);
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(code, 0, "err={}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    let verbose: Vec<_> = out.lines().filter(|l| l.starts_with("0\t")).collect();
    assert_eq!(verbose.len(), 1, "out={out}");
    assert!(!out.contains("1\t"), "out={out}");
}

#[test]
fn dirs_s_and_l_round_trip() {
    let start = std::env::current_dir().unwrap();
    let a = scratch("sa");
    let b = scratch("sb");
    let file = scratch("stackfile");
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    let script = format!(
        "pushd {}\npushd {}\ndirs -S {}\ndirs -c\ndirs -L {}\ndirs -v\n",
        a.display(),
        b.display(),
        file.display(),
        file.display()
    );
    let mut stdin = Cursor::new(script);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    let _ = std::env::set_current_dir(&start);
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    let _ = fs::remove_file(&file);
    assert_eq!(code, 0, "err={}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("0\t"), "out={out}");
    assert!(out.contains("1\t"), "out={out}");
    assert!(out.contains("2\t"), "out={out}");
}

#[test]
fn pushd_plus_rotates_stack() {
    let start = std::env::current_dir().unwrap();
    let a = scratch("ra");
    let b = scratch("rb");
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    let script = format!(
        "pushd {}\npushd {}\npushd +1\npwd\n",
        a.display(),
        b.display()
    );
    let mut stdin = Cursor::new(script);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    let cwd_after = std::env::current_dir().unwrap();
    let expected = a.canonicalize().unwrap_or(a.clone());
    let actual = cwd_after.canonicalize().unwrap_or(cwd_after.clone());
    let _ = std::env::set_current_dir(&start);
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    assert_eq!(code, 0, "err={}", String::from_utf8_lossy(&stderr));
    assert_eq!(actual, expected);
}

#[test]
fn popd_plus_drops_middle_entry() {
    let start = std::env::current_dir().unwrap();
    let a = scratch("pa");
    let b = scratch("pb");
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    let script = format!(
        "pushd {}\npushd {}\npopd +1\ndirs -v\n",
        a.display(),
        b.display()
    );
    let mut stdin = Cursor::new(script);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    let _ = std::env::set_current_dir(&start);
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
    assert_eq!(code, 0, "err={}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("0\t"), "out={out}");
    assert!(!out.contains("2\t"), "out={out}");
}

#[test]
fn dirs_unknown_option_prints_usage() {
    let mut env = ShellEnvironment::from_map(BTreeMap::new());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = status_of(
        builtins::try_run(
            &["dirs".into(), "-x".into()],
            &mut env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap(),
    );
    assert_eq!(code, 1);
    assert!(String::from_utf8(stderr).unwrap().contains("Usage: dirs"));
}
