//! `|` pipeline execution — OS pipes between simple-command stages.
//!
//! Builtins in a multi-stage pipeline use subshell semantics (cloned env;
//! `exit` does not kill the parent shell). Status is the last stage’s status.

use super::{
    abandon_children, build_external_command, report_spawn_failure, run_builtin_status,
    wait_children, CommandResult,
};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::Pipeline;

use std::io::{self, Write};
use std::process::{Child, Stdio};

/// Multi-stage `|` pipeline. Status is the last stage’s status.
pub(super) fn execute_piped_stages(
    pipeline: &Pipeline<'_>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let stages: Vec<Vec<String>> = pipeline
        .commands
        .iter()
        .map(|command| {
            command
                .argv
                .iter()
                .map(|word| (*word).to_string())
                .collect()
        })
        .collect();

    let mut children: Vec<Child> = Vec::new();
    let mut prev_stdout: Option<std::process::ChildStdout> = None;
    let mut buffered_out: Option<Vec<u8>> = None;
    let last_index = stages.len() - 1;

    for (index, stage) in stages.iter().enumerate() {
        let is_last = index == last_index;
        let Some(name) = stage.first().map(String::as_str) else {
            abandon_children(&mut children);
            return Ok(CommandResult::Status(0));
        };

        if builtins::is_builtin(name) {
            // Discard unused stdin from the previous external stage.
            if let Some(mut reader) = prev_stdout.take() {
                let _ = io::copy(&mut reader, &mut io::sink());
            }
            let _ = buffered_out.take();

            let mut env_clone = shell_env.clone();
            if is_last {
                let status =
                    run_builtin_status(stage, &mut env_clone, last_status, stdout, stderr)?;
                // Reap prior external stages; pipeline status is the builtin’s.
                let _ = wait_children(&mut children)?;
                return Ok(CommandResult::Status(status));
            }

            let mut buffer = Vec::new();
            let _ = run_builtin_status(stage, &mut env_clone, last_status, &mut buffer, stderr)?;
            buffered_out = Some(buffer);
            continue;
        }

        let mut command = build_external_command(stage, shell_env);
        if let Some(buffer) = buffered_out.take() {
            command.stdin(Stdio::piped());
            if !is_last {
                command.stdout(Stdio::piped());
            }
            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(err) => {
                    abandon_children(&mut children);
                    return Ok(CommandResult::Status(report_spawn_failure(
                        name, &err, stderr,
                    )?));
                }
            };
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&buffer)?;
            }
            prev_stdout = child.stdout.take();
            children.push(child);
            continue;
        }

        if let Some(stdout_pipe) = prev_stdout.take() {
            command.stdin(stdout_pipe);
        }
        if !is_last {
            command.stdout(Stdio::piped());
        }

        match command.spawn() {
            Ok(mut child) => {
                prev_stdout = child.stdout.take();
                children.push(child);
            }
            Err(err) => {
                abandon_children(&mut children);
                return Ok(CommandResult::Status(report_spawn_failure(
                    name, &err, stderr,
                )?));
            }
        }
    }

    // Pipeline ended on an external (or empty builtin buffer edge).
    let _ = buffered_out.take();
    let _ = prev_stdout.take();
    let status = wait_children(&mut children)?;
    Ok(CommandResult::Status(status))
}
