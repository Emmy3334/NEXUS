//! Intensive post-refactor regression scenarios (combinatorial shell behavior).

mod scenarios;

use nexus::repl;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Mutex;

pub(super) static CWD_LOCK: Mutex<()> = Mutex::new(());

pub(super) fn run_script(input: &str) -> (u8, String, String) {
    let mut stdin = Cursor::new(input);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = repl::run(&mut stdin, &mut stdout, &mut stderr, false).unwrap();
    (
        code,
        String::from_utf8(stdout).unwrap(),
        String::from_utf8(stderr).unwrap(),
    )
}

pub(super) fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "nexus_adv_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ))
}

pub(super) fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
    CWD_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
