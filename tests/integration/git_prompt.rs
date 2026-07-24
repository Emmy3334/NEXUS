//! Classic and Powerlevel10k-inspired primary prompts.

use nexus::env::ShellEnvironment;
use nexus::repl::{format_primary, format_primary_with, PromptContext};
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

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()
        .filter(|s| s.success())
        .is_some()
}

#[test]
fn bare_prompt_outside_git_repo() {
    let _cwd = crate::cwd_lock::lock();
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
    let _cwd = crate::cwd_lock::lock();
    if !git_available() {
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
    let _cwd = crate::cwd_lock::lock();
    if !git_available() {
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

#[test]
fn powerlevel_theme_includes_user_and_prompt_char() {
    let mut env = ShellEnvironment::capture();
    env.set("NEXUS_PROMPT_STYLE", "powerlevel10k");
    env.set("NEXUS_PROMPT_ICONS", "ascii");
    env.set_local("user", "tester");
    env.set_local("home", "/tmp");
    env.set_local("cwd", "/tmp");
    let ctx = PromptContext::from_env(&env, 0);
    let prompt = format_primary_with(&ctx);
    assert!(prompt.contains("tester"));
    assert!(prompt.contains('>'));
    assert!(prompt.contains("\x1b["));
}

#[test]
fn powerlevel_error_status_uses_red_prompt_char() {
    let mut env = ShellEnvironment::capture();
    env.set("NEXUS_PROMPT_STYLE", "powerlevel10k");
    env.set("NEXUS_PROMPT_ICONS", "ascii");
    env.set_local("user", "tester");
    env.set_local("cwd", "/tmp");
    let ok = format_primary_with(&PromptContext::from_env(&env, 0));
    let err = format_primary_with(&PromptContext::from_env(&env, 1));
    assert!(ok.contains("\x1b[34m"));
    assert!(err.contains("\x1b[31m"));
}

#[test]
fn powerlevel_git_clean_repo_shows_branch() {
    let _cwd = crate::cwd_lock::lock();
    if !git_available() {
        return;
    }
    let root = temp_dir("p10k_clean");
    if !init_repo(&root) {
        let _ = fs::remove_dir_all(&root);
        return;
    }
    fs::write(root.join("README"), "hi\n").unwrap();
    assert!(git(&root, &["add", "README"]));
    assert!(git(&root, &["commit", "-m", "init"]));

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();
    let mut env = ShellEnvironment::capture();
    env.seed_specials();
    env.set("NEXUS_PROMPT_STYLE", "powerlevel10k");
    env.set("NEXUS_PROMPT_ICONS", "ascii");
    let prompt = format_primary_with(&PromptContext::from_env(&env, 0));
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert!(prompt.contains("main"));
    assert!(!prompt.contains("main*"));
}
