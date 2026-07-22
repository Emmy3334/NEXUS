//! Tests for `pushd` / `popd` / `dirs`.

use nexus::repl;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_dirs_{name}_{}", std::process::id()))
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
