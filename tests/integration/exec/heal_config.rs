//! Heal order / image / quiet / catch-all / env-pass configuration.

use super::common::test_env;
use nexus::heal::{
    attach_default_backends, container_heal_allowed, heal_catch_all, image_for, parse_order,
    quiet_from, resolve_settings, Backend, EnvPass,
};

#[test]
fn parse_order_default_tokens() {
    assert_eq!(
        parse_order("wasm,kube,docker"),
        vec![Backend::Wasm, Backend::Kube, Backend::Docker]
    );
}

#[test]
fn parse_order_skips_unknown_and_empty_falls_back() {
    assert_eq!(
        parse_order("wasm,nope,docker"),
        vec![Backend::Wasm, Backend::Docker]
    );
    assert_eq!(
        parse_order("???"),
        vec![Backend::Wasm, Backend::Kube, Backend::Docker]
    );
}

#[test]
fn resolve_prefers_local_heal_order_and_image() {
    let mut env = test_env();
    env.set_local("heal_order", "docker");
    env.set_local("heal_image", "busybox:latest");
    let settings = resolve_settings(&env);
    assert_eq!(settings.order, vec![Backend::Docker]);
    assert_eq!(settings.image, "busybox:latest");
}

#[test]
fn quiet_from_local_and_default() {
    let mut env = test_env();
    assert!(!quiet_from(&env));
    env.set_local("heal_quiet", "1");
    assert!(quiet_from(&env));
}

#[test]
fn parse_order_aliases_kubernetes() {
    assert_eq!(parse_order("k8s,wasm"), vec![Backend::Kube, Backend::Wasm]);
    assert_eq!(
        parse_order("kubernetes, docker "),
        vec![Backend::Kube, Backend::Docker]
    );
}

#[test]
fn attach_default_wasm_only_order() {
    let mut env = test_env();
    env.set_local("heal_order", "wasm");
    attach_default_backends(&mut env);
    assert_eq!(env.healers.len(), 1);
}

#[test]
fn catch_all_off_by_default() {
    let env = test_env();
    assert!(!heal_catch_all(&env));
    assert!(!resolve_settings(&env).catch_all);
}

#[test]
fn catch_all_enabled_from_local() {
    let mut env = test_env();
    env.set_local("heal_catch_all", "1");
    assert!(heal_catch_all(&env));
    assert!(resolve_settings(&env).catch_all);
}

#[test]
fn image_for_maps_known_commands() {
    let base = "alpine:3.20";
    assert_eq!(image_for("python3", base), "python:3.12-alpine");
    assert_eq!(image_for("node", base), "node:22-alpine");
    assert_eq!(image_for("npm", base), "node:22-alpine");
    assert_eq!(image_for("/usr/bin/ruby", base), "ruby:3.3-alpine");
    assert_eq!(image_for("ls", base), base);
}

#[test]
fn container_heal_skips_typos_unless_catch_all() {
    let mut env = test_env();
    assert!(!container_heal_allowed("sdn", &env));
    assert!(!container_heal_allowed("npde", &env));
    assert!(container_heal_allowed("npx", &env));
    env.set_local("heal_catch_all", "1");
    assert!(container_heal_allowed("sdn", &env));
}

#[test]
fn env_pass_defaults_to_none() {
    let env = test_env();
    assert_eq!(EnvPass::from_shell(&env), EnvPass::None);
    assert_eq!(resolve_settings(&env).env_pass, "none");
}

#[test]
fn env_pass_parse_all_and_allowlist() {
    assert_eq!(EnvPass::parse("*"), EnvPass::All);
    assert_eq!(EnvPass::parse("all"), EnvPass::All);
    assert_eq!(EnvPass::parse("none"), EnvPass::None);
    let allow = EnvPass::parse("FOO, BAR");
    assert!(allow.allows("FOO"));
    assert!(allow.allows("BAR"));
    assert!(!allow.allows("BAZ"));
    // Name may be listed; loader-path scrub still drops PATH at forward time.
    assert!(EnvPass::parse("PATH").allows("PATH"));
}

#[test]
fn env_pass_prefers_local_over_default() {
    let mut env = test_env();
    env.set_local("heal_env", "FOO,BAR");
    assert_eq!(resolve_settings(&env).env_pass, "BAR,FOO");
    env.set_local("heal_env", "*");
    assert_eq!(resolve_settings(&env).env_pass, "all");
}
