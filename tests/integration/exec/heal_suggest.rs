//! Heal-aware “did you mean?” after classic not-found.

use super::common::test_env;
use nexus::exec::execute_external;
use nexus::heal::{CommandResolver, ResolverChain};

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;

struct Decline;

impl CommandResolver for Decline {
    fn try_heal(
        &self,
        _argv: &[String],
        _shell_env: &mut nexus::env::ShellEnvironment,
        _stdout: &mut dyn Write,
        _stderr: &mut dyn Write,
    ) -> io::Result<Option<u8>> {
        Ok(None)
    }
}

fn scratch_bin(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nexus_suggest_{}_{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, b"#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    dir
}

#[test]
fn suggests_nearby_path_command_and_keeps_127() {
    let dir = scratch_bin("hellox");
    let mut env = test_env();
    env.set("PATH", dir.to_string_lossy().as_ref());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(&["helo"], &mut env, &mut stdout, &mut stderr).unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(code, 127);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Command not found"), "err={err}");
    assert!(err.contains("did you mean"), "err={err}");
    assert!(err.contains("hellox"), "err={err}");
    assert!(!err.contains("heal tried"), "err={err}");
}

#[test]
fn heal_tip_when_backends_declined() {
    let mut env = test_env();
    env.set_local("heal_order", "wasm");
    env.healers = ResolverChain::from_resolvers(vec![Arc::new(Decline)]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(
        &["nexus_tip_missing_zzz"],
        &mut env,
        &mut stdout,
        &mut stderr,
    )
    .unwrap();
    assert_eq!(code, 127);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Command not found"));
    assert!(err.contains("heal tried"), "err={err}");
}

#[test]
fn heal_quiet_suppresses_suggestions_and_tip() {
    let dir = scratch_bin("hellox");
    let mut env = test_env();
    env.set("PATH", dir.to_string_lossy().as_ref());
    env.set_local("heal_quiet", "1");
    env.healers = ResolverChain::from_resolvers(vec![Arc::new(Decline)]);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = execute_external(&["helo"], &mut env, &mut stdout, &mut stderr).unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(code, 127);
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("Command not found"));
    assert!(!err.contains("did you mean"), "err={err}");
    assert!(!err.contains("heal tried"), "err={err}");
}
