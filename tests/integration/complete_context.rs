//! Context-aware Tab completion (git / interpreter / @docker).

use nexus::repl::complete;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn complete_at(line: &str) -> (String, Vec<String>) {
    let mut buffer = line.to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor);
    (buffer, matches)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_complete_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn git_checkout_lists_ambiguous_branches() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("git");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (buf, matches) = complete_at("git checkout ma");
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert_eq!(matches, vec!["main".to_string(), "master".to_string()]);
    assert_eq!(buf, "git checkout ma");
}

#[test]
fn git_checkout_unique_branch_prefix() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("git2");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("develop"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (buf, matches) = complete_at("git checkout ma");
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout main");
}

#[test]
fn git_checkout_dash_b_still_completes_branches() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("git3");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("develop"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (buf, matches) = complete_at("git checkout -b ma");
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout -b main");
}

#[test]
fn python_prefers_py_files() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("py");
    fs::write(root.join("app.py"), "").unwrap();
    fs::write(root.join("readme.md"), "").unwrap();
    fs::write(root.join("util.rb"), "").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (buf, matches) = complete_at("python ");
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert!(matches.is_empty());
    assert_eq!(buf, "python app.py");
}

#[test]
fn ruby_prefers_rb_files() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("rb");
    fs::write(root.join("app.py"), "").unwrap();
    fs::write(root.join("script.rb"), "").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (buf, matches) = complete_at("ruby ");
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert!(matches.is_empty());
    assert_eq!(buf, "ruby script.rb");
}

#[test]
fn docker_logs_completes_running_container_when_present() {
    if !Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return;
    }
    let name = format!("nexus_complete_{}", std::process::id());
    let status = Command::new("docker")
        .args([
            "run",
            "-d",
            "--rm",
            "--name",
            &name,
            "alpine:3.20",
            "sleep",
            "30",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        return;
    }

    let (buf, matches) = complete_at("@docker logs nexus_complete_");
    let _ = Command::new("docker")
        .args(["rm", "-f", &name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    assert!(matches.is_empty());
    assert_eq!(buf, format!("@docker logs {name}"));
}
