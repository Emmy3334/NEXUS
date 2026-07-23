//! Hook spawn failures into the heal chain before classic reporting.

use super::report_spawn_failure;
use super::suggest;
use crate::env::ShellEnvironment;

use std::ffi::OsStr;
use std::io::{self, Write};

/// After a failed spawn: try healers on `NotFound`, else report as usual.
pub fn after_spawn_failure(
    argv: &[String],
    err: &io::Error,
    shell_env: &mut ShellEnvironment,
    stdout: &mut dyn Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    after_spawn_failure_stdin(argv, err, shell_env, None, stdout, stderr)
}

/// Like [`after_spawn_failure`], forwarding finite stdin bytes into healers.
pub fn after_spawn_failure_stdin(
    argv: &[String],
    err: &io::Error,
    shell_env: &mut ShellEnvironment,
    stdin: Option<&[u8]>,
    stdout: &mut dyn Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let program = argv.first().map(String::as_str).unwrap_or("");
    tracing::debug!(argv0 = program, error = %err, "heal after_spawn_failure");
    if err.kind() != io::ErrorKind::NotFound {
        return report_spawn_failure(program, err, stderr);
    }
    let chain = shell_env.healers.clone();
    let healers_tried = !chain.is_empty();
    if let Some(code) = chain.try_heal(argv, shell_env, stdin, stdout, stderr)? {
        tracing::info!(argv0 = program, status = code, "heal chain handled");
        return Ok(code);
    }
    let code = report_spawn_failure(program, err, stderr)?;
    if code == 127 {
        suggest::write_after_not_found(program, shell_env, healers_tried, stderr)?;
    }
    Ok(code)
}

/// Same as [`after_spawn_failure`] when argv is only known as `OsStr` slices.
pub fn after_spawn_failure_os(
    argv: &[impl AsRef<OsStr>],
    err: &io::Error,
    shell_env: &mut ShellEnvironment,
    stdout: &mut dyn Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let owned: Vec<String> = argv
        .iter()
        .map(|a| a.as_ref().to_string_lossy().into_owned())
        .collect();
    after_spawn_failure(&owned, err, shell_env, stdout, stderr)
}
