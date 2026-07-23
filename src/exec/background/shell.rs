//! Launch builtins, subshells, and pipelines in an isolated NEXUS process.

use super::super::redirect::HeredocState;
use super::super::CommandResult;
use crate::env::ShellEnvironment;
use crate::parse::{CommandList, Pipeline, PipelineCommand, RedirectKind};

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub(super) fn spawn_pipeline(
    pipeline: &Pipeline<'_>,
    shell_env: &mut ShellEnvironment,
    heredocs: &mut HeredocState,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let source = super::super::render::render_pipeline(pipeline);
    let script = build_script(pipeline, &source, heredocs, stderr)?;
    let mut command = Command::new(nexus_path()?);
    command
        .env_clear()
        .envs(shell_env.iter())
        .stdin(Stdio::piped());
    // Avoid `pre_exec`/`setpgid` here: on macOS, large Rust binaries can hang in
    // dyld when `setpgid` runs between fork and exec. Set the pgrp in the parent.
    let mut child = command.spawn()?;
    let _ = crate::jobs::detach_background(&child);
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(script.as_bytes())?;
    }
    let (id, pgid) = shell_env
        .jobs
        .add(source, vec![child], crate::jobs::JobState::Running);
    writeln!(stderr, "[{id}] {pgid}")?;
    Ok(CommandResult::Status(0))
}

fn nexus_path() -> io::Result<PathBuf> {
    // Integration tests: Cargo sets this to the package binary it just built.
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_nexus") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }
    let current = std::env::current_exe()?;
    if current.file_stem().is_some_and(|name| name == "nexus") {
        return Ok(current);
    }
    let candidate = current
        .parent()
        .and_then(|path| path.parent())
        .map(|path| path.join("nexus"));
    candidate
        .filter(|path| path.is_file())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "nexus executable not found"))
}

fn build_script(
    pipeline: &Pipeline<'_>,
    source: &str,
    heredocs: &mut HeredocState,
    stderr: &mut impl Write,
) -> io::Result<String> {
    let mut delimiters = Vec::new();
    collect_pipeline_heredocs(pipeline, &mut delimiters);
    let mut script = format!("{source}\n");
    for delimiter in delimiters {
        let body = match heredocs.take_next(stderr)? {
            Ok(body) => body,
            Err(_) => return Ok("false\n".to_owned()),
        };
        script.push_str(&body);
        if !body.ends_with('\n') {
            script.push('\n');
        }
        script.push_str(delimiter);
        script.push('\n');
    }
    Ok(script)
}

fn collect_pipeline_heredocs<'a>(pipeline: &'a Pipeline<'a>, out: &mut Vec<&'a str>) {
    for command in &pipeline.commands {
        if let PipelineCommand::Subshell { list, .. } = command {
            collect_list_heredocs(list, out);
        }
        out.extend(
            command
                .redirects()
                .iter()
                .filter(|redirect| redirect.kind == RedirectKind::Heredoc)
                .map(|redirect| redirect.path),
        );
    }
}

fn collect_list_heredocs<'a>(list: &'a CommandList<'a>, out: &mut Vec<&'a str>) {
    for pipeline in &list.pipelines {
        collect_pipeline_heredocs(pipeline, out);
    }
}
