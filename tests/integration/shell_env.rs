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

#[test]
fn seed_specials_from_exported_and_cwd() {
    let mut env = ShellEnvironment::default();
    env.set("HOME", "/tmp/home");
    env.set("USER", "nexus");
    env.set("TERM", "xterm");
    env.seed_specials();
    assert_eq!(env.get_local("home"), Some("/tmp/home"));
    assert_eq!(env.get_local("user"), Some("nexus"));
    assert_eq!(env.get_local("term"), Some("xterm"));
    assert!(env.get_local("cwd").is_some());
}
