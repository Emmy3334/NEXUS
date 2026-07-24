//! Repeated Tab cycles through ambiguous completion matches.

use nexus::env::ShellEnvironment;
use nexus::repl::{complete_or_cycle, CompleteCtx, CompleteCycle};
use std::fs;
use std::path::PathBuf;

fn cycle_at(line: &str, cycle: &mut Option<CompleteCycle>) -> (String, Vec<String>) {
    let env = ShellEnvironment::default();
    let names = env.var_names();
    let ctx = CompleteCtx {
        var_names: &names,
        registry: env.comp_registry(),
    };
    let mut buffer = line.to_owned();
    let mut cursor = buffer.len();
    let matches = complete_or_cycle(&mut buffer, &mut cursor, &ctx, cycle);
    (buffer, matches)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_cycle_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn first_tab_lists_then_cycles_and_wraps() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("git");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let mut cycle = None;
    let (buf, matches) = cycle_at("git checkout ma", &mut cycle);
    assert_eq!(matches, vec!["main".to_string(), "master".to_string()]);
    assert_eq!(buf, "git checkout ma");

    let (buf, matches) = cycle_at(&buf, &mut cycle);
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout main");

    let (buf, matches) = cycle_at(&buf, &mut cycle);
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout master");

    let (buf, matches) = cycle_at(&buf, &mut cycle);
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout main");

    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn unique_match_does_not_start_cycle() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("unique");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("develop"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let mut cycle = None;
    let (buf, matches) = cycle_at("git checkout ma", &mut cycle);
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout main");
    assert!(cycle.is_none());

    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn clearing_cycle_relists_on_next_tab() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("invalidate");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let mut cycle = None;
    let (buf, _) = cycle_at("git checkout ma", &mut cycle);
    let (buf, _) = cycle_at(&buf, &mut cycle);
    assert_eq!(buf, "git checkout main");

    cycle = None;
    let (buf, matches) = cycle_at("git checkout ma", &mut cycle);
    assert_eq!(matches, vec!["main".to_string(), "master".to_string()]);
    assert_eq!(buf, "git checkout ma");

    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn common_prefix_then_cycle() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("prefix");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let mut cycle = None;
    let (buf, matches) = cycle_at("git checkout m", &mut cycle);
    assert_eq!(matches, vec!["main".to_string(), "master".to_string()]);
    assert_eq!(buf, "git checkout ma");

    let (buf, matches) = cycle_at(&buf, &mut cycle);
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout main");

    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stale_cycle_token_restarts_complete() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("stale");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let mut cycle = None;
    let _ = cycle_at("git checkout ma", &mut cycle);
    assert!(cycle.is_some());

    let mut buffer = "git checkout max".to_owned();
    let mut cursor = buffer.len();
    let env = ShellEnvironment::default();
    let names = env.var_names();
    let ctx = CompleteCtx {
        var_names: &names,
        registry: env.comp_registry(),
    };
    let matches = complete_or_cycle(&mut buffer, &mut cursor, &ctx, &mut cycle);
    assert!(matches.is_empty());
    assert_eq!(buffer, "git checkout max");

    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn arrow_menu_moves_highlight_and_accept_applies() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("menu");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let mut cycle = None;
    let (buf, matches) = cycle_at("git checkout ma", &mut cycle);
    assert_eq!(matches.len(), 2);
    let c = cycle.as_mut().unwrap();
    assert_eq!(c.highlight, 0);
    c.move_down();
    assert_eq!(c.highlight, 1);
    c.move_up();
    assert_eq!(c.highlight, 0);
    c.move_up();
    assert_eq!(c.highlight, 1);

    let mut buffer = buf;
    let mut cursor = buffer.len();
    c.accept(&mut buffer, &mut cursor);
    assert_eq!(buffer, "git checkout master");

    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);
}
