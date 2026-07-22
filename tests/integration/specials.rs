//! Special aliases (`precmd`, `cwdcmd`) and `ignoreeof`.

use nexus::env::ShellEnvironment;
use nexus::repl;
use nexus::specials;
use std::io::Cursor;

#[test]
fn ignoreeof_unset_exits_immediately() {
    let env = ShellEnvironment::from_map(Default::default());
    let mut streak = 0;
    let mut stderr = Vec::new();
    assert!(specials::allow_exit_on_eof(&env, &mut streak, &mut stderr).unwrap());
    assert_eq!(streak, 0);
    assert!(stderr.is_empty());
}

#[test]
fn ignoreeof_empty_keeps_prompting() {
    let mut env = ShellEnvironment::from_map(Default::default());
    env.set_local("ignoreeof", "");
    let mut streak = 0;
    let mut stderr = Vec::new();
    assert!(!specials::allow_exit_on_eof(&env, &mut streak, &mut stderr).unwrap());
    assert_eq!(streak, 1);
    assert!(String::from_utf8(stderr).unwrap().contains("exit"));
}

#[test]
fn ignoreeof_two_exits_on_second() {
    let mut env = ShellEnvironment::from_map(Default::default());
    env.set_local("ignoreeof", "2");
    let mut streak = 0;
    let mut stderr = Vec::new();
    assert!(!specials::allow_exit_on_eof(&env, &mut streak, &mut stderr).unwrap());
    assert!(specials::allow_exit_on_eof(&env, &mut streak, &mut stderr).unwrap());
    assert_eq!(streak, 2);
}

#[test]
fn precmd_runs_before_interactive_command() {
    let mut stdin = Cursor::new("alias precmd set hooked=yes\nset\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, true).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("hooked=yes"), "{out}");
}

#[test]
fn cwdcmd_runs_after_cd() {
    let dir = std::env::temp_dir().join(format!("nexus_cwdcmd_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let input = format!("alias cwdcmd set hopped=yes\ncd {}\nset\n", dir.display());
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let out = String::from_utf8(stdout).unwrap();
    assert!(out.contains("hopped=yes"), "{out}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ignoreeof_repl_requires_two_eofs() {
    let mut stdin = Cursor::new("set ignoreeof=2\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, true).unwrap();
    assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let err = String::from_utf8(stderr).unwrap();
    assert!(err.contains("exit"), "{err}");
}
