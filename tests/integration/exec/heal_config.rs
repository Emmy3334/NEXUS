//! Heal order / image / quiet configuration.

use super::common::test_env;
use nexus::heal::{attach_default_backends, parse_order, quiet_from, resolve_settings, Backend};

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
fn attach_default_wasm_only_order() {
    let mut env = test_env();
    env.set_local("heal_order", "wasm");
    attach_default_backends(&mut env);
    assert_eq!(env.healers.len(), 1);
}
