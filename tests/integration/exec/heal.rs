//! Self-heal resolver seam: missing commands can be recovered by backends.

use super::common::test_env;
use nexus::env::ShellEnvironment;
use nexus::exec::execute_external;
use nexus::heal::{CommandResolver, ResolverChain};

use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct FixedStatus {
    status: u8,
    hits: AtomicUsize,
}

impl CommandResolver for FixedStatus {
    fn try_heal(
        &self,
        argv: &[String],
        _shell_env: &mut ShellEnvironment,
        stdout: &mut dyn Write,
        _stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        self.hits.fetch_add(1, Ordering::SeqCst);
        writeln!(
            stdout,
            "healed:{}",
            argv.first().map(String::as_str).unwrap_or("")
        )?;
        Ok(Some(self.status))
    }
}

struct Decline;

impl CommandResolver for Decline {
    fn try_heal(
        &self,
        _argv: &[String],
        _shell_env: &mut ShellEnvironment,
        _stdout: &mut dyn Write,
        _stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        Ok(None)
    }
}

#[test]
fn empty_healers_keep_classic_not_found() {
    let mut env = test_env();
    assert!(env.healers.is_empty());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(
        &["nexus_no_such_heal_cmd"],
        &mut env,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(code, 127);
    let message = String::from_utf8(stderr).unwrap();
    assert!(message.contains("Command not found"));
    assert!(stdout.is_empty());
}

#[test]
fn resolver_runs_on_not_found_and_returns_status() {
    let healer = Arc::new(FixedStatus {
        status: 42,
        hits: AtomicUsize::new(0),
    });
    let mut env = test_env();
    env.healers = ResolverChain::from_resolvers(vec![healer.clone()]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(
        &["nexus_missing_for_heal"],
        &mut env,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(code, 42);
    assert_eq!(healer.hits.load(Ordering::SeqCst), 1);
    assert!(stderr.is_empty());
    assert_eq!(
        String::from_utf8(stdout).unwrap(),
        "healed:nexus_missing_for_heal\n"
    );
}

#[test]
fn declining_resolver_falls_through_to_127() {
    let mut env = test_env();
    env.healers = ResolverChain::from_resolvers(vec![Arc::new(Decline)]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code =
        execute_external(&["nexus_still_missing"], &mut env, &mut stdout, &mut stderr).unwrap();
    assert_eq!(code, 127);
    assert!(String::from_utf8(stderr)
        .unwrap()
        .contains("Command not found"));
}
