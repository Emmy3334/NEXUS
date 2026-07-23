//! Opt-in child rlimit (`rlimit` / `NEXUS_RLIMIT`).

use nexus::env::ShellEnvironment;
use nexus::harden;
use std::collections::BTreeMap;

fn test_env() -> ShellEnvironment {
    ShellEnvironment::from_map(BTreeMap::new())
}

#[test]
fn rlimit_off_by_default() {
    let env = test_env();
    assert!(!harden::rlimit_enabled(&env));
}

#[test]
fn rlimit_enabled_via_local() {
    let mut env = test_env();
    env.set_local("rlimit", "1");
    assert!(harden::rlimit_enabled(&env));
}
