//! Opt-in `trusted_bin` / `NEXUS_TRUSTED_BIN` absolute-path allowlist.

use nexus::env::ShellEnvironment;
use nexus::exec::{execute_list, CommandResult};
use nexus::harden;
use nexus::lex::tokenize_into;
use nexus::parse::parse_line;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn test_env() -> ShellEnvironment {
    let mut map = BTreeMap::new();
    if let Ok(path) = std::env::var("PATH") {
        map.insert("PATH".into(), path);
    }
    ShellEnvironment::from_map(map)
}

fn run(source: &str, env: &mut ShellEnvironment) -> (CommandResult, String) {
    let mut tokens = Vec::new();
    tokenize_into(source, &mut tokens).unwrap();
    let list = parse_line(source, &tokens).unwrap().unwrap();
    let mut argv = Vec::new();
    let mut stderr = Vec::new();
    let result = execute_list(
        &list,
        &mut argv,
        env,
        0,
        Vec::new(),
        &mut std::io::empty(),
        &mut Vec::new(),
        &mut stderr,
    )
    .unwrap();
    (result, String::from_utf8(stderr).unwrap())
}

#[test]
fn trusted_bin_off_by_default() {
    let env = test_env();
    assert!(harden::trusted_bin_allowlist(&env).is_none());
}

#[test]
fn trusted_bin_disabled_with_off() {
    let mut env = test_env();
    env.set_local("trusted_bin", "0");
    assert!(harden::trusted_bin_allowlist(&env).is_none());
}

#[test]
fn allowlist_parses_absolute_dirs() {
    let mut env = test_env();
    env.set_local("trusted_bin", "/bin:/usr/bin:relative:/tmp/evil");
    let list = harden::trusted_bin_allowlist(&env).expect("on");
    assert_eq!(
        list,
        vec![
            PathBuf::from("/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/tmp/evil")
        ]
    );
}

#[test]
fn denies_path_outside_allowlist() {
    let mut env = test_env();
    env.set_local("trusted_bin", "/usr/sbin");
    let (result, err) = run("/bin/echo hi", &mut env);
    assert_eq!(result, CommandResult::Status(126));
    assert!(err.contains("not in trusted_bin allowlist"), "err={err}");
}

#[test]
fn allows_prefix_directory() {
    let mut env = test_env();
    env.set_local("trusted_bin", "/bin");
    let (result, err) = run("/bin/echo ok", &mut env);
    assert_eq!(result, CommandResult::Status(0), "err={err}");
}

#[test]
fn allows_exact_file() {
    let mut env = test_env();
    env.set_local("trusted_bin", "/bin/echo");
    let (result, err) = run("/bin/echo ok", &mut env);
    assert_eq!(result, CommandResult::Status(0), "err={err}");
}

#[test]
fn denies_path_resolved_via_path() {
    let mut env = test_env();
    env.set_local("trusted_bin", "/usr/sbin");
    // `echo` resolves under /bin on macOS/Linux — outside /usr/sbin.
    let (result, err) = run("echo hi", &mut env);
    assert_eq!(result, CommandResult::Status(126), "err={err}");
    assert!(err.contains("not in trusted_bin allowlist"), "err={err}");
}

#[test]
fn check_helper_allows_unresolved_bare_name() {
    let mut env = test_env();
    env.set_local("trusted_bin", "/bin");
    assert!(harden::check_trusted("missingcmd", "missingcmd", &env).is_ok());
}
