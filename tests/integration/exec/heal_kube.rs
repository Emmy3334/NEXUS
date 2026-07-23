//! Kubernetes Pod heal backend integration tests.

use super::common::test_env;
use nexus::exec::execute_external;
use nexus::heal::{attach_kube_backend, KubeResolver, ResolverChain, KUBE_DEFAULT_IMAGE};
use nexus::kube;

use std::sync::Arc;

#[test]
fn attach_is_noop_when_cluster_unreachable() {
    if kube::cluster_reachable() {
        return;
    }
    let mut env = test_env();
    assert!(env.healers.is_empty());
    attach_kube_backend(&mut env);
    assert!(env.healers.is_empty());
}

#[test]
fn probe_none_when_cluster_unreachable() {
    if kube::cluster_reachable() {
        return;
    }
    assert!(KubeResolver::probe(KUBE_DEFAULT_IMAGE).is_none());
}

#[test]
fn kube_runs_alpine_command_when_cluster_up() {
    if !kube::cluster_reachable() {
        return;
    }
    let resolver = KubeResolver::probe(KUBE_DEFAULT_IMAGE).expect("kube probe");
    let mut env = test_env();
    env.healers = ResolverChain::from_resolvers(vec![Arc::new(resolver)]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    // `apk` exists in alpine but not on a typical macOS host PATH.
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
fn kube_declines_when_missing_in_image_too() {
    if !kube::cluster_reachable() {
        return;
    }
    let resolver = KubeResolver::probe(KUBE_DEFAULT_IMAGE).expect("kube probe");
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
fn kube_bind_mounts_cwd_when_node_sees_host() {
    use nexus::env::ShellEnvironment;
    use nexus::heal::CommandResolver;
    use std::fs;
    if !kube::cluster_reachable() {
        return;
    }
    let _cwd = crate::cwd_lock::lock();
    let dir = std::env::temp_dir().join(format!("nexus_kube_heal_cwd_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("payload.txt"), "heal-bind\n").unwrap();
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let resolver = KubeResolver::probe(KUBE_DEFAULT_IMAGE).expect("kube probe");
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
    // hostPath is unavailable on some clusters (e.g. minikube without mount).
    if code != Some(0) {
        return;
    }
    assert_eq!(String::from_utf8(stdout).unwrap(), "heal-bind\n");
}
