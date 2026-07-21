//! Integration tests for [`nexus::env::ShellEnvironment`].

use nexus::env::ShellEnvironment;

#[test]
fn set_get_unset() {
    let mut env = ShellEnvironment::default();
    env.set("FOO", "bar");
    assert_eq!(env.get("FOO"), Some("bar"));
    assert!(env.unset("FOO"));
    assert_eq!(env.get("FOO"), None);
}

#[test]
fn lookup_prefers_local_over_exported() {
    let mut env = ShellEnvironment::default();
    env.set("FOO", "exported");
    env.set_local("FOO", "local");
    assert_eq!(env.lookup("FOO"), Some("local"));
    assert_eq!(env.get("FOO"), Some("exported"));
}

#[test]
fn lookup_falls_back_to_exported() {
    let mut env = ShellEnvironment::default();
    env.set("BAR", "from_env");
    assert_eq!(env.lookup("BAR"), Some("from_env"));
    assert_eq!(env.lookup("MISSING"), None);
}
