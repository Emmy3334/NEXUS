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
