//! Soft host isolation for non-Wasm `sandbox` targets.

use crate::env::ShellEnvironment;
use crate::exec::exit_status_code;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const KEEP_ENV: &[&str] = &["PATH", "TERM", "LANG", "LC_ALL", "LC_CTYPE", "TZ"];

/// Run `argv` in a temp HOME with a filtered environment (no heal backends).
pub fn run_host(
    argv: &[String],
    shell_env: &ShellEnvironment,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };
    let work = make_work()?;
    let status = spawn_isolated(program, args, &work, shell_env, stdout, stderr);
    let _ = fs::remove_dir_all(&work);
    status
}

fn spawn_isolated(
    program: &str,
    args: &[String],
    work: &Path,
    shell_env: &ShellEnvironment,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(work)
        .env_clear()
        .envs(filtered_env(shell_env))
        .env("HOME", work)
        .env("TMPDIR", work)
        .env("NEXUS_SANDBOX", "1");
    match command.output() {
        Ok(out) => {
            stdout.write_all(&out.stdout)?;
            stderr.write_all(&out.stderr)?;
            Ok(exit_status_code(out.status))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            writeln!(stderr, "{program}: Command not found.")?;
            Ok(127)
        }
        Err(err) => Err(err),
    }
}

fn filtered_env(shell_env: &ShellEnvironment) -> Vec<(String, String)> {
    KEEP_ENV
        .iter()
        .filter_map(|key| {
            let value = shell_env
                .lookup(key)
                .map(str::to_owned)
                .or_else(|| env::var(key).ok())?;
            Some(((*key).to_owned(), value))
        })
        .collect()
}

fn make_work() -> io::Result<PathBuf> {
    let dir = env::temp_dir().join(format!("nexus_sandbox_{}", std::process::id()));
    let dir = dir.join(format!("{}", uniq()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn uniq() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::Relaxed)
}
