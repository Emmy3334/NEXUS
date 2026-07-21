//! Spawn an external for a background simple command.

use super::super::io::ExecIo;
use super::super::redirect::{open_redirect_files, HeredocState, RedirectFiles, StdinSource};
use super::super::{build_external_command, report_spawn_failure, CommandResult};
use crate::env::ShellEnvironment;
use crate::parse::Redirect;

use std::io::{self, BufRead, Write};
use std::process::{Child, Command, Stdio};

pub(super) fn spawn_simple<I: BufRead, O: Write, E: Write>(
    argv: &[String],
    redirects: &[Redirect<'_>],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    let files = match open_redirect_files(
        redirects,
        heredocs,
        shell_env,
        last_status,
        io.stdin,
        io.stderr,
    )? {
        Ok(files) => files,
        Err(code) => return Ok(CommandResult::Status(code)),
    };
    let mut command = build_external_command(argv, shell_env);
    apply_bg_stdio(&mut command, files);
    crate::jobs::prepare_background_group(&mut command);
    match command.spawn() {
        Ok(child) => register_job(argv, vec![child], shell_env, io.stderr),
        Err(err) => Ok(CommandResult::Status(report_spawn_failure(
            &argv[0], &err, io.stderr,
        )?)),
    }
}

fn apply_bg_stdio(command: &mut Command, files: RedirectFiles) {
    match files.stdin {
        Some(StdinSource::File(file)) => {
            command.stdin(Stdio::from(file));
        }
        Some(StdinSource::Bytes(_)) | None => {
            command.stdin(Stdio::null());
        }
    }
    match files.stdout {
        Some(file) => {
            command.stdout(Stdio::from(file));
        }
        None => {
            command.stdout(Stdio::null());
        }
    }
}

fn register_job(
    argv: &[String],
    children: Vec<Child>,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if let Some(child) = children.first() {
        let _ = crate::jobs::detach_background(child);
    }
    let command = argv.join(" ");
    let (id, pgid) = shell_env
        .jobs
        .add(command, children, crate::jobs::JobState::Running);
    writeln!(stderr, "[{id}] {pgid}")?;
    Ok(CommandResult::Status(0))
}
