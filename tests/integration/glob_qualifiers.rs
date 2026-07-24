//! Integration tests for zsh-style glob qualifiers `(.)` `(/)` `(*)`.

use nexus::env::ShellEnvironment;
use nexus::expand::expand_word_for_exec;
use nexus::glob::expand_globs;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn test_env() -> ShellEnvironment {
    let path = std::env::var("PATH").unwrap_or_default();
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), path);
    ShellEnvironment::from_map(map)
}

fn scratch_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_gqual_{name}_{}", std::process::id()))
}

fn expand_pattern(pattern: &str) -> Vec<String> {
    let word = expand_word_for_exec(pattern, &mut ShellEnvironment::default(), 0).unwrap();
    expand_globs(&word)
}

#[test]
fn qualifier_regular_files_only() {
    let cwd = crate::cwd_lock::RestoreCwd::new();
    let dir = scratch_dir("dot");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    cwd.chdir(&dir).unwrap();

    assert_eq!(expand_pattern("*(.)"), vec!["a.txt".to_owned()]);
    assert_eq!(
        expand_pattern("*"),
        vec!["a.txt".to_owned(), "sub".to_owned()]
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn qualifier_directories_only() {
    let cwd = crate::cwd_lock::RestoreCwd::new();
    let dir = scratch_dir("slash");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    cwd.chdir(&dir).unwrap();

    assert_eq!(expand_pattern("*(/)"), vec!["sub".to_owned()]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn literal_parens_without_active_glob() {
    let mut env = test_env();
    let word = expand_word_for_exec("foo(.)", &mut env, 0).unwrap();
    assert!(!word.has_active_glob());
    assert_eq!(expand_globs(&word), vec!["foo(.)".to_owned()]);
}

#[cfg(unix)]
#[test]
fn qualifier_executables_only() {
    use std::os::unix::fs::PermissionsExt;

    let cwd = crate::cwd_lock::RestoreCwd::new();
    let dir = scratch_dir("exec");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("run.sh"), "#!/bin/sh\n").unwrap();
    fs::write(dir.join("plain.txt"), "").unwrap();
    let mut perms = fs::metadata(dir.join("run.sh")).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(dir.join("run.sh"), perms).unwrap();
    cwd.chdir(&dir).unwrap();

    assert_eq!(expand_pattern("*(*)"), vec!["run.sh".to_owned()]);

    let _ = fs::remove_dir_all(&dir);
}
