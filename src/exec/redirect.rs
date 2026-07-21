//! File and heredoc redirections for simple commands and pipeline stages.
//!
//! File / heredoc redirects override a pipe on the same fd. Heredoc bodies are
//! collected from the shell input stream after parse (see
//! [`collect_heredoc_bodies`]).

use super::{exit_status_code, report_spawn_failure, CommandResult};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::expand;
use crate::parse::{CommandList, Redirect, RedirectKind};

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

pub(super) enum StdinSource {
    File(File),
    /// Heredoc (or other in-memory) stdin payload.
    Bytes(Vec<u8>),
}

pub(super) struct RedirectFiles {
    pub(super) stdin: Option<StdinSource>,
    pub(super) stdout: Option<File>,
}

/// Cursor over pre-collected heredoc bodies (one per `<<`, left-to-right).
///
/// Takes ownership of each body when applied so we do not clone the payload.
pub(super) struct HeredocState {
    bodies: Vec<String>,
    index: usize,
}

impl HeredocState {
    pub(super) fn new(bodies: Vec<String>) -> Self {
        Self { bodies, index: 0 }
    }

    fn take_next(&mut self, stderr: &mut impl Write) -> io::Result<Result<String, u8>> {
        if self.index >= self.bodies.len() {
            writeln!(stderr, "nexus: missing heredoc body")?;
            return Ok(Err(1));
        }
        let body = std::mem::take(&mut self.bodies[self.index]);
        self.index += 1;
        Ok(Ok(body))
    }
}

/// Read heredoc bodies for every `<<` in `list`, left-to-right.
///
/// Each body is the input lines up to (but not including) a line whose content
/// equals the (quote- and `$`-expanded) delimiter. On EOF before the delimiter, the
/// partial body is kept and a warning is written to `stderr`.
///
/// Returns `Ok(Err(code))` when quote expansion of a delimiter fails.
pub fn collect_heredoc_bodies(
    list: &CommandList<'_>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    input: &mut impl BufRead,
    stderr: &mut impl Write,
) -> io::Result<Result<Vec<String>, u8>> {
    let mut bodies = Vec::new();
    let mut line = String::new();

    for pipeline in &list.pipelines {
        for command in &pipeline.commands {
            for redirect in &command.redirects {
                if redirect.kind == RedirectKind::Heredoc {
                    let delimiter =
                        match expand::expand_word_for_exec(redirect.path, shell_env, last_status) {
                            Ok(delimiter) => delimiter,
                            Err(err) => {
                                writeln!(stderr, "{}", err.message())?;
                                return Ok(Err(1));
                            }
                        };
                    bodies.push(read_heredoc_body(input, &delimiter, &mut line, stderr)?);
                }
            }
        }
    }

    Ok(Ok(bodies))
}

fn read_heredoc_body(
    input: &mut impl BufRead,
    delimiter: &str,
    line: &mut String,
    stderr: &mut impl Write,
) -> io::Result<String> {
    let mut body = String::new();
    loop {
        line.clear();
        let bytes = input.read_line(line)?;
        if bytes == 0 {
            writeln!(
                stderr,
                "nexus: warning: here-document delimited by end-of-file (wanted `{delimiter}`)"
            )?;
            break;
        }
        let without_ending = line.trim_end_matches(['\n', '\r']);
        if without_ending == delimiter {
            break;
        }
        body.push_str(line);
    }
    Ok(body)
}

/// Open / materialize redirect targets. On failure, writes to `stderr` and returns `Err(1)`.
pub(super) fn open_redirect_files(
    redirects: &[Redirect<'_>],
    heredocs: &mut HeredocState,
    shell_env: &ShellEnvironment,
    last_status: u8,
    stderr: &mut impl Write,
) -> io::Result<Result<RedirectFiles, u8>> {
    let mut stdin = None;
    let mut stdout = None;

    for redirect in redirects {
        let path = match expand::expand_word_for_exec(redirect.path, shell_env, last_status) {
            Ok(path) => path,
            Err(err) => {
                writeln!(stderr, "{}", err.message())?;
                return Ok(Err(1));
            }
        };

        match redirect.kind {
            RedirectKind::Read => match File::open(&path) {
                Ok(file) => stdin = Some(StdinSource::File(file)),
                Err(err) => {
                    writeln!(stderr, "{path}: {err}")?;
                    return Ok(Err(1));
                }
            },
            RedirectKind::Write => match File::create(&path) {
                Ok(file) => stdout = Some(file),
                Err(err) => {
                    writeln!(stderr, "{path}: {err}")?;
                    return Ok(Err(1));
                }
            },
            RedirectKind::Append => {
                match OpenOptions::new().create(true).append(true).open(&path) {
                    Ok(file) => stdout = Some(file),
                    Err(err) => {
                        writeln!(stderr, "{path}: {err}")?;
                        return Ok(Err(1));
                    }
                }
            }
            RedirectKind::Heredoc => match heredocs.take_next(stderr)? {
                Ok(body) => stdin = Some(StdinSource::Bytes(body.into_bytes())),
                Err(code) => return Ok(Err(code)),
            },
        }
    }

    Ok(Ok(RedirectFiles { stdin, stdout }))
}

pub(super) fn apply_stdout_for_stage(
    command: &mut Command,
    stdout_file: Option<File>,
    is_last: bool,
) {
    if let Some(file) = stdout_file {
        command.stdout(Stdio::from(file));
    } else if !is_last {
        command.stdout(Stdio::piped());
    }
}

/// Run one simple command, applying file / heredoc redirects when present.
pub(super) fn execute_simple(
    argv: &[String],
    redirects: &[Redirect<'_>],
    heredocs: &mut HeredocState,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if redirects.is_empty() {
        return super::execute_command(argv, shell_env, last_status, stdout, stderr);
    }

    let files = match open_redirect_files(redirects, heredocs, shell_env, last_status, stderr)? {
        Ok(files) => files,
        Err(code) => return Ok(CommandResult::Status(code)),
    };

    if let Some(name) = argv.first().map(String::as_str) {
        if builtins::is_builtin(name) {
            return execute_builtin_with_files(argv, files, shell_env, last_status, stdout, stderr);
        }
    }

    Ok(CommandResult::Status(execute_external_with_files(
        argv, shell_env, files, stderr,
    )?))
}

fn execute_builtin_with_files(
    argv: &[String],
    files: RedirectFiles,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    // Builtins don't consume stdin yet; drop any stdin redirect (path already validated).
    drop(files.stdin);

    let result = match files.stdout {
        Some(mut file) => builtins::try_run(argv, shell_env, last_status, &mut file, stderr)?,
        None => builtins::try_run(argv, shell_env, last_status, stdout, stderr)?,
    };

    Ok(result
        .map(CommandResult::from)
        .unwrap_or(CommandResult::Status(0)))
}

fn execute_external_with_files(
    argv: &[String],
    shell_env: &ShellEnvironment,
    files: RedirectFiles,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };

    let mut command = Command::new(program);
    command.args(args).env_clear().envs(shell_env.iter());

    let stdin_bytes = match files.stdin {
        Some(StdinSource::File(file)) => {
            command.stdin(Stdio::from(file));
            None
        }
        Some(StdinSource::Bytes(bytes)) => {
            command.stdin(Stdio::piped());
            Some(bytes)
        }
        None => None,
    };
    if let Some(file) = files.stdout {
        command.stdout(Stdio::from(file));
    }

    match stdin_bytes {
        None => match command.status() {
            Ok(status) => Ok(exit_status_code(status)),
            Err(err) => Ok(report_spawn_failure(program, &err, stderr)?),
        },
        Some(bytes) => match command.spawn() {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(&bytes)?;
                }
                let status = child.wait()?;
                Ok(exit_status_code(status))
            }
            Err(err) => Ok(report_spawn_failure(program, &err, stderr)?),
        },
    }
}
