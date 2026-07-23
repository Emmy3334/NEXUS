//! Opening / materializing redirect targets (`<`, `>`, `>>`, `<<`).

use super::heredoc::HeredocState;
use crate::env::ShellEnvironment;
use crate::exec::StdoutMode;
use crate::expand;
use crate::parse::{Redirect, RedirectKind};

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

pub(in crate::exec) enum StdinSource {
    File(File),
    /// Heredoc (or other in-memory) stdin payload.
    Bytes(Vec<u8>),
}

pub(in crate::exec) struct RedirectFiles {
    pub(in crate::exec) stdin: Option<StdinSource>,
    pub(in crate::exec) stdout: Option<File>,
}

/// Open / materialize redirect targets. On failure, writes to `stderr` and returns `Err(1)`.
pub(in crate::exec) fn open_redirect_files(
    redirects: &[Redirect<'_>],
    heredocs: &mut HeredocState,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> io::Result<Result<RedirectFiles, u8>> {
    let mut files = RedirectFiles {
        stdin: None,
        stdout: None,
    };

    for redirect in redirects {
        if redirect.kind == RedirectKind::Heredoc {
            match heredocs.take_next(stderr)? {
                Ok(body) => files.stdin = Some(StdinSource::Bytes(body.into_bytes())),
                Err(code) => return Ok(Err(code)),
            }
            continue;
        }

        let path = match resolve_redirect_path(redirect, shell_env, last_status, stdin, stderr)? {
            Ok(path) => path,
            Err(code) => return Ok(Err(code)),
        };
        if let Err(code) = apply_redirect(redirect.kind, &path, &mut files, stderr)? {
            return Ok(Err(code));
        }
    }

    Ok(Ok(files))
}

fn resolve_redirect_path(
    redirect: &Redirect<'_>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> io::Result<Result<String, u8>> {
    let mut fields = Vec::new();
    let snap = shell_env.clone();
    let mut capture =
        |body: &str| crate::exec::capture_command_output(body, &snap, last_status, stdin, stderr);
    match expand::expand_word_fields_into(
        redirect.path,
        shell_env,
        last_status,
        &mut fields,
        &mut capture,
    ) {
        Ok(()) => {
            let word = fields.into_iter().next().unwrap_or_default();
            match crate::glob::expand_globs_one(&word) {
                Ok(path) => Ok(Ok(path)),
                Err(_) => {
                    writeln!(stderr, "{}: Ambiguous redirect.", word.as_str())?;
                    Ok(Err(1))
                }
            }
        }
        Err(err) => {
            writeln!(stderr, "{}", err.message())?;
            Ok(Err(1))
        }
    }
}

/// Open the file for a `<` / `>` / `>>` redirect and wire it into `files`.
/// (`Heredoc` is handled separately in [`open_redirect_files`], in-memory.)
fn apply_redirect(
    kind: RedirectKind,
    path: &str,
    files: &mut RedirectFiles,
    stderr: &mut impl Write,
) -> io::Result<Result<(), u8>> {
    match open_file(open_target(kind, path), path, stderr)? {
        Ok(file) => {
            assign_target(kind, file, files);
            Ok(Ok(()))
        }
        Err(code) => Ok(Err(code)),
    }
}

fn open_target(kind: RedirectKind, path: &str) -> io::Result<File> {
    match kind {
        RedirectKind::Read => File::open(path),
        RedirectKind::Append => OpenOptions::new().create(true).append(true).open(path),
        RedirectKind::Write | RedirectKind::Heredoc => File::create(path),
    }
}

fn assign_target(kind: RedirectKind, file: File, files: &mut RedirectFiles) {
    if kind == RedirectKind::Read {
        files.stdin = Some(StdinSource::File(file));
    } else {
        files.stdout = Some(file);
    }
}

fn open_file(
    result: io::Result<File>,
    path: &str,
    stderr: &mut impl Write,
) -> io::Result<Result<File, u8>> {
    match result {
        Ok(file) => Ok(Ok(file)),
        Err(err) => {
            writeln!(stderr, "{path}: {err}")?;
            Ok(Err(1))
        }
    }
}

pub(in crate::exec) fn apply_stdout_for_stage(
    command: &mut Command,
    stdout_file: Option<File>,
    is_last: bool,
    stdout_mode: StdoutMode,
) {
    if let Some(file) = stdout_file {
        command.stdout(Stdio::from(file));
    } else if !is_last || stdout_mode == StdoutMode::Capture {
        command.stdout(Stdio::piped());
    }
}
