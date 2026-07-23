//! Context-aware Tab completion (git / interpreter / @docker).

use nexus::env::ShellEnvironment;
use nexus::repl::complete;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn complete_at(line: &str) -> (String, Vec<String>) {
    let mut buffer = line.to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor, &ShellEnvironment::default());
    (buffer, matches)
}

#[test]
fn git_subcommand_completes_checkout_prefix() {
    let (buf, matches) = complete_at("git checko");
    assert!(matches.is_empty());
    assert_eq!(buf, "git checkout");
}

#[test]
fn git_subcommand_lists_ambiguous_ch() {
    let (buf, matches) = complete_at("git ch");
    assert!(matches.iter().any(|m| m == "checkout"));
    assert!(matches.iter().any(|m| m == "cherry" || m == "cherry-pick"));
    assert!(buf.starts_with("git ch"));
}

#[test]
fn git_space_lists_subcommands() {
    let (_, matches) = complete_at("git ");
    assert!(matches.iter().any(|m| m == "status"));
    assert!(matches.iter().any(|m| m == "checkout"));
}

#[test]
fn cargo_subcommand_completes_unique_prefix() {
    let (buf, matches) = complete_at("cargo clip");
    assert!(matches.is_empty());
    assert_eq!(buf, "cargo clippy");
}

#[test]
fn docker_subcommand_lists_ambiguous_lo() {
    let (_, matches) = complete_at("docker lo");
    assert!(matches.iter().any(|m| m == "logs"));
    assert!(matches.iter().any(|m| m == "login" || m == "logout"));
}

#[test]
fn npm_subcommand_completes_install_prefix() {
    let (buf, matches) = complete_at("npm instal");
    assert!(matches.is_empty());
    assert_eq!(buf, "npm install");
}

#[test]
fn unregistered_command_does_not_use_git_verbs() {
    let (buf, _) = complete_at("mytool checko");
    assert_ne!(buf, "mytool checkout");
}

#[test]
fn helm_subcommand_completes_unique_prefix() {
    let (buf, matches) = complete_at("helm upgra");
    assert!(matches.is_empty());
    assert_eq!(buf, "helm upgrade");
}

#[test]
fn brew_subcommand_lists_in() {
    let (_, matches) = complete_at("brew in");
    assert!(matches.iter().any(|m| m == "info" || m == "install"));
}

#[test]
fn terraform_subcommand_completes_unique_prefix() {
    let (buf, matches) = complete_at("terraform valid");
    assert!(matches.is_empty());
    assert_eq!(buf, "terraform validate");
}

#[test]
fn systemctl_subcommand_completes_unique_prefix() {
    let (buf, matches) = complete_at("systemctl resta");
    assert!(matches.is_empty());
    assert_eq!(buf, "systemctl restart");
}

#[test]
fn git_commit_flag_completes_amend() {
    let (buf, matches) = complete_at("git commit --am");
    assert!(matches.is_empty());
    assert_eq!(buf, "git commit --amend");
}

#[test]
fn git_log_lists_long_flags() {
    let (_, matches) = complete_at("git log --");
    assert!(matches.iter().any(|m| m == "--oneline"));
    assert!(matches.iter().any(|m| m == "--stat"));
}

#[test]
fn git_merge_completes_branches() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("git_merge");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (buf, matches) = complete_at("git merge ma");
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(root);

    assert_eq!(matches, vec!["main".to_string(), "master".to_string()]);
    assert_eq!(buf, "git merge ma");
}

#[test]
fn git_checkout_dash_dash_lists_flags_not_branches() {
    let (_, matches) = complete_at("git checkout --");
    assert!(matches.iter().any(|m| m == "--orphan" || m == "--detach"));
    assert!(!matches.iter().any(|m| m == "main"));
}

#[test]
fn kubectl_get_completes_resource_kind() {
    let (buf, matches) = complete_at("kubectl get dep");
    assert!(matches.is_empty());
    assert_eq!(buf, "kubectl get deployments");
}

#[test]
fn kubectl_get_with_namespace_flag_still_completes_kind() {
    let (buf, matches) = complete_at("kubectl -n default get po");
    assert!(matches.is_empty());
    assert_eq!(buf, "kubectl -n default get pods");
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
fn git_checkout_extends_common_prefix() {
    let _cwd = crate::cwd_lock::lock();
    let root = temp_dir("git_prefix");
    let heads = root.join(".git/refs/heads");
    fs::create_dir_all(&heads).unwrap();
    fs::write(heads.join("main"), "abc\n").unwrap();
    fs::write(heads.join("master"), "def\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();

    let (buf, matches) = complete_at("git checkout m");
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
