//! Canonical builtin names shared by recognition and Tab complete.

/// Sorted builtin `argv[0]` names (binary-search / Tab prefix match).
pub const NAMES: &[&str] = &[
    ".", ":", "@", "@docker", "@kube", "alias", "bg", "bindkey", "cd", "dirs", "disown", "doctor",
    "echo", "env", "exit", "false", "fg", "heal", "history", "jobs", "local", "popd", "pushd",
    "repeat", "return", "sandbox", "set", "setenv", "source", "true", "typeset", "unalias",
    "unset", "unsetenv", "where", "which",
];

/// Whether `name` is a shell builtin.
#[must_use]
pub fn is_builtin(name: &str) -> bool {
    NAMES.binary_search(&name).is_ok()
}
