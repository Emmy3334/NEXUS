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
    let matches = complete_matches_for_test("my-spec", &env);
    let hit = matches
        .iter()
        .find(|m| m.value == "my-special-target")
        .expect("history word");
    assert_eq!(hit.tag, Tag::History);
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
