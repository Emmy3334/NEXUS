//! Hook spawn failures into the heal chain before classic reporting.

use super::report_spawn_failure;
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
    let program = argv.first().map(String::as_str).unwrap_or("");
    if err.kind() != io::ErrorKind::NotFound {
        return report_spawn_failure(program, err, stderr);
    }
    let chain = shell_env.healers.clone();
    if let Some(code) = chain.try_heal(argv, shell_env, stdout, stderr)? {
        return Ok(code);
    }
    report_spawn_failure(program, err, stderr)
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
