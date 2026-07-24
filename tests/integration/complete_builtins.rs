//! Tab completion uses the shared builtin name list.

use nexus::builtins::{is_builtin, NAMES};
use nexus::env::ShellEnvironment;
use nexus::repl::complete;

fn empty_env() -> ShellEnvironment {
    ShellEnvironment::default()
}

#[test]
fn names_are_sorted_for_binary_search() {
    let mut sorted = NAMES.to_vec();
    sorted.sort_unstable();
    assert_eq!(NAMES, sorted.as_slice());
}

#[test]
fn every_canonical_name_is_recognized() {
    for name in NAMES {
        assert!(is_builtin(name), "{name}");
    }
    assert!(!is_builtin("not-a-builtin"));
}

#[test]
fn tab_suggests_cloud_and_extra_builtins() {
    for (prefix, name) in [
        ("@dock", "@docker"),
        ("@kub", "@kube"),
        ("sandb", "sandbox"),
        ("bindk", "bindkey"),
        ("pushd", "pushd"),
        ("popd", "popd"),
        ("dirs", "dirs"),
        ("which", "which"),
        ("where", "where"),
        ("repeat", "repeat"),
        ("types", "typeset"),
        ("echo", "echo"),
        ("true", "true"),
        ("false", "false"),
    ] {
        let mut buffer = prefix.to_owned();
        let mut cursor = buffer.len();
        let matches = complete(&mut buffer, &mut cursor, &empty_env());
        let hit = buffer.starts_with(name) || matches.iter().any(|m| m == name);
        assert!(
            hit,
            "prefix {prefix}: buffer={buffer:?} matches={matches:?}"
        );
    }
}
