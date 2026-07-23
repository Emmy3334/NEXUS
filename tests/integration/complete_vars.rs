//! `$VAR` / `${VAR}` Tab completion.

use nexus::env::ShellEnvironment;
use nexus::repl::complete;
use std::collections::BTreeMap;

fn env_with(vars: &[(&str, &str)], locals: &[(&str, &str)]) -> ShellEnvironment {
    let map = vars
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect::<BTreeMap<_, _>>();
    let mut env = ShellEnvironment::from_map(map);
    for (k, v) in locals {
        env.set_local(*k, *v);
    }
    env
}

#[test]
fn tab_completes_dollar_prefix() {
    let env = env_with(&[("HOME", "/tmp"), ("HOST", "box")], &[]);
    let mut buffer = "$HO".to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor, &env);
    assert!(
        buffer.starts_with("$HOME") || matches.iter().any(|m| m == "$HOME"),
        "buffer={buffer:?} matches={matches:?}"
    );
    assert!(
        buffer.starts_with("$HOST")
            || matches.iter().any(|m| m == "$HOST")
            || matches.iter().any(|m| m == "$HOME"),
        "expected HOME/HOST among results"
    );
}

#[test]
fn tab_completes_braced_form() {
    let env = env_with(&[("HOME", "/tmp")], &[]);
    let mut buffer = "echo ${HO".to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor, &env);
    let hit = buffer.contains("${HOME}") || matches.iter().any(|m| m == "${HOME}");
    assert!(hit, "buffer={buffer:?} matches={matches:?}");
}

#[test]
fn tab_lists_locals_and_exports() {
    let env = env_with(&[("EXPORTED", "1")], &[("LOCALVAR", "2")]);
    let mut buffer = "$".to_owned();
    let mut cursor = 1;
    let matches = complete(&mut buffer, &mut cursor, &env);
    let joined = format!("{buffer} {}", matches.join(" "));
    assert!(joined.contains("$EXPORTED"), "{joined}");
    assert!(joined.contains("$LOCALVAR"), "{joined}");
}

#[test]
fn non_dollar_tokens_unchanged() {
    let env = env_with(&[("HOME", "/tmp")], &[]);
    let mut buffer = "hea".to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor, &env);
    let hit = buffer.starts_with("heal") || matches.iter().any(|m| m == "heal");
    assert!(hit, "buffer={buffer:?} matches={matches:?}");
}
