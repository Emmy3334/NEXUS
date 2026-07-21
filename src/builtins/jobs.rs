//! `jobs`, `fg`, and `bg` builtins.

use crate::env::ShellEnvironment;
use crate::jobs::JobSpec;

use std::io::{self, Write};

/// `jobs` — list running background jobs.
pub(super) fn jobs_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() > 1 {
        writeln!(stderr, "jobs: Too many arguments.")?;
        return Ok(1);
    }
    shell_env.jobs.print_jobs(stdout)?;
    Ok(0)
}

/// `fg [job]` — wait for a background job in the foreground.
pub(super) fn fg_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let spec = match parse_spec(argv, stderr)? {
        Ok(spec) => spec,
        Err(code) => return Ok(code),
    };
    match shell_env.jobs.foreground(spec, stderr)? {
        Ok(status) => Ok(status),
        Err(err) => {
            writeln!(stderr, "fg: {}", err.message())?;
            Ok(1)
        }
    }
}

/// `bg [job]` — continue a job in the background.
pub(super) fn bg_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let spec = match parse_spec(argv, stderr)? {
        Ok(spec) => spec,
        Err(code) => return Ok(code),
    };
    match shell_env.jobs.background(spec, stderr)? {
        Ok(()) => Ok(0),
        Err(err) => {
            writeln!(stderr, "bg: {}", err.message())?;
            Ok(1)
        }
    }
}

fn parse_spec(argv: &[String], stderr: &mut impl Write) -> io::Result<Result<JobSpec, u8>> {
    match argv.get(1).map(String::as_str) {
        None => Ok(Ok(JobSpec::Current)),
        Some(raw) => match parse_job_id(raw) {
            Ok(id) => Ok(Ok(JobSpec::Id(id))),
            Err(()) => {
                writeln!(stderr, "No such job.")?;
                Ok(Err(1))
            }
        },
    }
}

fn parse_job_id(raw: &str) -> Result<usize, ()> {
    let digits = raw.strip_prefix('%').unwrap_or(raw);
    if digits.is_empty() {
        return Err(());
    }
    digits.parse().map_err(|_| ())
}
