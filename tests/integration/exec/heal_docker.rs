//! Docker heal backend integration tests.

use super::common::test_env;
use nexus::exec::execute_external;
use nexus::heal::{attach_docker_backend, DockerResolver, ResolverChain, DEFAULT_IMAGE};

use std::process::Command;
use std::sync::Arc;

fn docker_available() -> bool {
    Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[test]
fn attach_is_noop_when_docker_unreachable() {
    if docker_available() {
        return;
    }
    let mut env = test_env();
    assert!(env.healers.is_empty());
    attach_docker_backend(&mut env);
    assert!(env.healers.is_empty());
}

#[test]
fn probe_none_when_docker_unreachable() {
    if docker_available() {
        return;
    }
    assert!(DockerResolver::probe(DEFAULT_IMAGE).is_none());
}

#[test]
fn docker_runs_alpine_command_when_daemon_up() {
    if !docker_available() {
        return;
    }
    let resolver = DockerResolver::probe(DEFAULT_IMAGE).expect("docker probe");
    let mut env = test_env();
    env.healers = ResolverChain::from_resolvers(vec![Arc::new(resolver)]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    // `apk` is present in alpine but not on a typical macOS/Linux host PATH.
    let code = execute_external(&["apk", "--version"], &mut env, &mut stdout, &mut stderr)
        .expect("execute");
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(
        out.to_ascii_lowercase().contains("apk"),
        "unexpected stdout: {out:?}"
    );
}

#[test]
fn docker_declines_when_missing_in_image_too() {
    if !docker_available() {
        return;
    }
    let resolver = DockerResolver::probe(DEFAULT_IMAGE).expect("docker probe");
    let mut env = test_env();
    env.healers = ResolverChain::from_resolvers(vec![Arc::new(resolver)]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(
        &["nexus_not_in_alpine_zzz"],
        &mut env,
        &mut stdout,
        &mut stderr,
    )
    .expect("execute");
    assert_eq!(code, 127);
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Command not found"));
}

#[test]
fn docker_bind_mounts_cwd_for_host_files() {
    use nexus::env::ShellEnvironment;
    use nexus::heal::CommandResolver;
    use std::fs;
    if !docker_available() {
        return;
    }
    let _cwd = super::common::CWD_LOCK.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("nexus_heal_cwd_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("payload.txt"), "heal-bind\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let resolver = DockerResolver::probe(DEFAULT_IMAGE).expect("docker probe");
    let mut env = ShellEnvironment::from_map(Default::default());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = resolver.try_heal(
        &["cat".into(), "payload.txt".into()],
        &mut env,
        &mut stdout,
        &mut stderr,
    );
    let _ = std::env::set_current_dir(prev);
    let _ = fs::remove_dir_all(&dir);

    let code = result.expect("try_heal io");
    assert_eq!(code, Some(0), "stderr={}", String::from_utf8_lossy(&stderr));
    assert_eq!(String::from_utf8(stdout).unwrap(), "heal-bind\n");
}
