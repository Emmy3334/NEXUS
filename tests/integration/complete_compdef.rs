//! `compdef` / `compinit` / `compdump` completion registry.

use nexus::builtins::{self, BuiltinResult};
use nexus::env::{dump_path_is_safe, ShellEnvironment};
use nexus::repl::complete;

fn run(argv: &[&str], env: &mut ShellEnvironment) -> u8 {
    let argv: Vec<String> = argv.iter().map(|s| (*s).to_owned()).collect();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    match builtins::try_run(&argv, env, 0, &mut stdout, &mut stderr).unwrap() {
        Some(BuiltinResult::Status(code)) => code,
        other => panic!("unexpected builtin result: {other:?}"),
    }
}

fn complete_at(env: &ShellEnvironment, line: &str) -> (String, Vec<String>) {
    let mut buffer = line.to_owned();
    let mut cursor = buffer.len();
    let matches = complete(&mut buffer, &mut cursor, env);
    (buffer, matches)
}

#[test]
fn compdef_registers_first_verb_words() {
    let mut env = ShellEnvironment::default();
    assert_eq!(run(&["compdef", "mycmd", "foo", "bar"], &mut env), 0);
    let (buf, matches) = complete_at(&env, "mycmd fo");
    assert!(matches.is_empty());
    assert_eq!(buf, "mycmd foo");
    let (buf, matches) = complete_at(&env, "mycmd b");
    assert!(matches.is_empty());
    assert_eq!(buf, "mycmd bar");
    let (_, matches) = complete_at(&env, "mycmd ");
    assert!(matches.iter().any(|m| m == "foo"));
    assert!(matches.iter().any(|m| m == "bar"));
}

#[test]
fn curated_subcommand_wins_over_compdef() {
    let mut env = ShellEnvironment::default();
    assert_eq!(run(&["compdef", "git", "custom"], &mut env), 0);
    let (buf, _) = complete_at(&env, "git checko");
    assert_eq!(buf, "git checkout");
}

#[test]
fn dump_roundtrip_restores_completion() {
    let dir = std::env::temp_dir().join(format!(
        "nexus_compdef_{}_{}",
        std::process::id(),
        "roundtrip"
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dump = dir.join("comp.dump");

    let mut env = ShellEnvironment::default();
    env.set("HOME", dir.to_str().unwrap());
    assert_eq!(run(&["compdef", "mycmd", "alpha", "beta"], &mut env), 0);
    assert_eq!(
        run(&["compdump", "-d", dump.to_str().unwrap()], &mut env),
        0
    );

    assert_eq!(run(&["compdef", "-d", "mycmd"], &mut env), 0);
    assert_eq!(
        run(&["compinit", "-d", dump.to_str().unwrap()], &mut env),
        0
    );

    let (buf, _) = complete_at(&env, "mycmd al");
    assert_eq!(buf, "mycmd alpha");
}

#[test]
#[cfg(unix)]
fn world_writable_dump_is_refused() {
    use std::os::unix::fs::PermissionsExt;

    let dir =
        std::env::temp_dir().join(format!("nexus_compdef_{}_{}", std::process::id(), "unsafe"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dump = dir.join("unsafe.dump");

    let mut env = ShellEnvironment::default();
    assert_eq!(run(&["compdef", "mycmd", "foo"], &mut env), 0);
    assert_eq!(
        run(&["compdump", "-d", dump.to_str().unwrap()], &mut env),
        0
    );
    let mut perms = std::fs::metadata(&dump).unwrap().permissions();
    perms.set_mode(0o666);
    std::fs::set_permissions(&dump, perms).unwrap();
    assert!(!dump_path_is_safe(&dump).unwrap());

    assert_eq!(run(&["compdef", "-d", "mycmd"], &mut env), 0);
    assert_eq!(
        run(&["compinit", "-d", dump.to_str().unwrap()], &mut env),
        0
    );

    let (buf, _) = complete_at(&env, "mycmd f");
    assert_ne!(buf, "mycmd foo");
}

#[test]
fn compdef_delete_removes_words() {
    let mut env = ShellEnvironment::default();
    assert_eq!(run(&["compdef", "mycmd", "foo"], &mut env), 0);
    assert_eq!(run(&["compdef", "-d", "mycmd"], &mut env), 0);
    let (buf, _) = complete_at(&env, "mycmd f");
    assert_ne!(buf, "mycmd foo");
}

#[test]
fn compdef_usage_errors_return_one() {
    let mut env = ShellEnvironment::default();
    assert_eq!(run(&["compdef"], &mut env), 1);
    assert_eq!(run(&["compdef", "-d"], &mut env), 1);
}
