//! Parameter operators: `${var:-}`, `${var:+}`, `${#var}`, `#`/`##`/`%`/`%%`.

use nexus::env::ShellEnvironment;
use nexus::expand::expand_word_for_exec;
use std::collections::BTreeMap;

fn expand(raw: &str, env: &mut ShellEnvironment, status: u8) -> String {
    expand_word_for_exec(raw, env, status)
        .unwrap()
        .into_string()
}

fn env_with(pairs: &[(&str, &str)]) -> ShellEnvironment {
    let mut map = BTreeMap::new();
    for &(k, v) in pairs {
        map.insert(k.into(), v.into());
    }
    ShellEnvironment::from_map(map)
}

#[test]
fn default_when_unset_or_empty() {
    let mut env = env_with(&[("HIT", "yes")]);
    assert_eq!(expand("${HIT:-no}", &mut env, 0), "yes");
    assert_eq!(expand("${MISS:-fallback}", &mut env, 0), "fallback");
    env.set_local("EMPTY", "");
    assert_eq!(expand("${EMPTY:-x}", &mut env, 0), "x");
}

#[test]
fn default_word_expands_nested() {
    let mut env = env_with(&[("OTHER", "nested")]);
    assert_eq!(expand("${MISS:-$OTHER}", &mut env, 0), "nested");
    assert_eq!(expand("${MISS:-${OTHER}}", &mut env, 0), "nested");
}

#[test]
fn alternate_when_set_nonempty() {
    let mut env = env_with(&[("HIT", "yes")]);
    assert_eq!(expand("${HIT:+alt}", &mut env, 0), "alt");
    assert_eq!(expand("${MISS:+alt}", &mut env, 0), "");
    env.set_local("EMPTY", "");
    assert_eq!(expand("${EMPTY:+alt}", &mut env, 0), "");
}

#[test]
fn length_of_value_and_argc() {
    let mut env = env_with(&[("NAME", "abcd")]);
    assert_eq!(expand("${#NAME}", &mut env, 0), "4");
    assert_eq!(expand("${#MISS}", &mut env, 0), "0");
    assert_eq!(expand("${#}", &mut env, 0), "0");
}

#[test]
fn strip_prefix_shortest_and_longest() {
    let mut env = env_with(&[("PATHLIKE", "foo/bar/baz")]);
    assert_eq!(expand("${PATHLIKE#*/}", &mut env, 0), "bar/baz");
    assert_eq!(expand("${PATHLIKE##*/}", &mut env, 0), "baz");
    assert_eq!(expand("${PATHLIKE#*}", &mut env, 0), "foo/bar/baz");
    assert_eq!(expand("${PATHLIKE##*}", &mut env, 0), "");
}

#[test]
fn strip_suffix_shortest_and_longest() {
    let mut env = env_with(&[("FILE", "archive.tar.gz")]);
    assert_eq!(expand("${FILE%.*}", &mut env, 0), "archive.tar");
    assert_eq!(expand("${FILE%%.*}", &mut env, 0), "archive");
}

#[test]
fn operators_work_inside_double_quotes() {
    let mut env = env_with(&[("A", "hi")]);
    assert_eq!(expand("\"${#A}:${MISS:-ok}\"", &mut env, 0), "2:ok");
}

#[test]
fn plain_braced_name_unchanged() {
    let mut env = env_with(&[("HOME", "/tmp")]);
    assert_eq!(expand("${HOME}", &mut env, 0), "/tmp");
}
