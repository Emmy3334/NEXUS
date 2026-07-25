//! Completion uses shell PATH / path jail and history-word frecency.

use nexus::env::ShellEnvironment;
use nexus::repl::{complete_matches_for_test, list_menu_lines_tagged, Match, Tag};
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nexus_cpath_{name}_{}", std::process::id()))
}

#[test]
fn path_commands_come_from_shell_path_not_process() {
    let dir = scratch("bin");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let bin = dir.join("nexusuniqcmd");
    fs::write(&bin, "#!/bin/sh\n").unwrap();
    let mut perms = fs::metadata(&bin).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&bin, perms).unwrap();

    let mut map = BTreeMap::new();
    map.insert("PATH".into(), dir.display().to_string());
    map.insert("path_jail".into(), "0".into());
    let env = ShellEnvironment::from_map(map);
    let matches = complete_matches_for_test("nexusuniq", &env);
    assert!(
        matches.iter().any(|m| m.value == "nexusuniqcmd"),
        "got {:?}",
        matches.iter().map(|m| &m.value).collect::<Vec<_>>()
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn history_words_appear_with_history_tag() {
    let mut env = ShellEnvironment::default();
    env.history.push("deploy my-special-target");
    env.history.push("echo my-special-target");
    // Argument position: history words are offered; first token stays command-only.
    let matches = complete_matches_for_test("echo my-spec", &env);
    let hit = matches
        .iter()
        .find(|m| m.value == "my-special-target")
        .expect("history word");
    assert_eq!(hit.tag, Tag::History);
}

#[test]
fn first_token_skips_cwd_files() {
    let _cwd = crate::cwd_lock::RestoreCwd::new();
    let dir = scratch("cwdfiles");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("min-local-file.pdf"), "").unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let mut map = BTreeMap::new();
    map.insert("PATH".into(), "/usr/bin".into());
    map.insert("path_jail".into(), "0".into());
    let mut env = ShellEnvironment::from_map(map);
    env.alias_set("minalias", "echo hi");
    env.function_set("minfunc", "echo fn");

    let matches = complete_matches_for_test("min", &env);
    assert!(
        matches.iter().all(|m| m.value != "min-local-file.pdf"),
        "cwd files must not appear on first token: {:?}",
        matches.iter().map(|m| &m.value).collect::<Vec<_>>()
    );
    assert!(matches.iter().any(|m| m.value == "minalias"));
    assert!(matches.iter().any(|m| m.value == "minfunc"));

    let arg_matches = complete_matches_for_test("cat min", &env);
    assert!(
        arg_matches.iter().any(|m| m.value == "min-local-file.pdf"),
        "argument position should offer cwd files"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cd_completes_directories_only() {
    let _cwd = crate::cwd_lock::lock();
    let dir = scratch("cddirs");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("target")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(dir.join("takefile.txt"), "").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let mut env = ShellEnvironment::default();
    // History word that only substring-matches the prefix must never leak in.
    env.history.push("cargo install");
    let matches = complete_matches_for_test("cd t", &env);
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(&dir);

    let values: Vec<&str> = matches.iter().map(|m| m.value.as_str()).collect();
    assert!(values.contains(&"target/"), "got {values:?}");
    assert!(values.contains(&"tests/"), "got {values:?}");
    assert!(
        !values.iter().any(|v| v.ends_with(".txt")),
        "no files: {values:?}"
    );
    assert!(!values.contains(&"install"), "no history: {values:?}");
}

#[test]
fn cd_prefix_match_only_no_substring() {
    let _cwd = crate::cwd_lock::lock();
    let dir = scratch("cdprefix");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("target")).unwrap();
    fs::create_dir_all(dir.join("StartupFiles")).unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let matches = complete_matches_for_test("cd ta", &ShellEnvironment::default());
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(&dir);

    let values: Vec<&str> = matches.iter().map(|m| m.value.as_str()).collect();
    assert_eq!(values, vec!["target/"], "prefix-only dirs: {values:?}");
}

#[test]
fn tagged_menu_shows_counts_and_history_section() {
    let matches = vec![
        Match {
            value: "echo".into(),
            description: None,
            tag: Tag::Commands,
            score: 100,
        },
        Match {
            value: "earlier".into(),
            description: None,
            tag: Tag::History,
            score: 80,
        },
    ];
    let lines = list_menu_lines_tagged(&matches, 0);
    assert!(lines.iter().any(|l| l == "-- commands (1) --"));
    assert!(lines.iter().any(|l| l == "-- history (1) --"));
}
