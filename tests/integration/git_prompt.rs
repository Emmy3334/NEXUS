//! Git-aware primary prompt (`$> [branch*] `).

use nexus::repl::format_primary;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_prompt_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(cwd: &PathBuf, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn init_repo(cwd: &PathBuf) -> bool {
    git(cwd, &["init", "-b", "main"])
        && git(cwd, &["config", "user.email", "nexus@test"])
        && git(cwd, &["config", "user.name", "nexus"])
}

#[test]
fn bare_prompt_outside_git_repo() {
    let root = temp_dir("bare");
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();
    let prompt = format_primary();
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);
    assert_eq!(prompt, "$> ");
}

#[test]
fn clean_branch_shows_in_brackets() {
    if Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()
        .filter(|s| s.success())
        .is_none()
    {
        return;
    }
    let root = temp_dir("clean");
    if !init_repo(&root) {
        let _ = fs::remove_dir_all(root);
        return;
    }
    fs::write(root.join("README"), "hi\n").unwrap();
    assert!(git(&root, &["add", "README"]));
    assert!(git(&root, &["commit", "-m", "init"]));

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();
    let prompt = format_primary();
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert_eq!(prompt, "$> [main] ");
}

#[test]
fn dirty_worktree_adds_asterisk() {
    if Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()
        .filter(|s| s.success())
        .is_none()
    {
        return;
    }
    let root = temp_dir("dirty");
    if !init_repo(&root) {
        let _ = fs::remove_dir_all(root);
        return;
    }
    fs::write(root.join("README"), "hi\n").unwrap();
    assert!(git(&root, &["add", "README"]));
    assert!(git(&root, &["commit", "-m", "init"]));
    fs::write(root.join("README"), "changed\n").unwrap();

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();
    let prompt = format_primary();
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert_eq!(prompt, "$> [main*] ");
}
