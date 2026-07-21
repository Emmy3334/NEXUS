//! `|` pipeline execution — OS pipes between simple-command stages.
//!
//! Builtins in a multi-stage pipeline use subshell semantics (cloned env;
//! `exit` does not kill the parent shell). Status is the last stage’s status.
//! File redirects on a stage override the pipe on that fd.

use super::redirect::{apply_stdout_for_stage, open_redirect_files};
use super::{
    abandon_children, build_external_command, report_spawn_failure, run_builtin_status,
    wait_children, CommandResult,
};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::{Pipeline, Redirect};

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
    let stages: Vec<(Vec<String>, &[Redirect<'_>])> = pipeline
        .commands
        .iter()
        .map(|command| {
            let argv = command
                .argv
                .iter()
                .map(|word| (*word).to_string())
                .collect();
            (argv, command.redirects.as_slice())
        })
        .collect();

    let mut children: Vec<Child> = Vec::new();
    let mut prev_stdout: Option<std::process::ChildStdout> = None;
    let mut buffered_out: Option<Vec<u8>> = None;
    let last_index = stages.len() - 1;

    for (index, (stage, redirects)) in stages.iter().enumerate() {
        let is_last = index == last_index;
        let Some(name) = stage.first().map(String::as_str) else {
            abandon_children(&mut children);
            return Ok(CommandResult::Status(0));
        };

        let files = match open_redirect_files(redirects, stderr)? {
            Ok(files) => files,
            Err(code) => {
                abandon_children(&mut children);
                let _ = wait_children(&mut children)?;
                return Ok(CommandResult::Status(code));
            }
        };

        if builtins::is_builtin(name) {
            if let Some(mut reader) = prev_stdout.take() {
                let _ = io::copy(&mut reader, &mut io::sink());
            }
            let _ = buffered_out.take();
            // Builtins don't consume stdin yet; drop the File (open already validated).
            drop(files.stdin);

            let mut env_clone = shell_env.clone();
            if is_last {
                let status = match files.stdout {
                    Some(mut file) => {
                        run_builtin_status(stage, &mut env_clone, last_status, &mut file, stderr)?
                    }
                    None => run_builtin_status(stage, &mut env_clone, last_status, stdout, stderr)?,
                };
                let _ = wait_children(&mut children)?;
                return Ok(CommandResult::Status(status));
            }

            match files.stdout {
                Some(mut file) => {
                    let _ =
                        run_builtin_status(stage, &mut env_clone, last_status, &mut file, stderr)?;
                }
                None => {
                    let mut buffer = Vec::new();
                    let _ = run_builtin_status(
                        stage,
                        &mut env_clone,
                        last_status,
                        &mut buffer,
                        stderr,
                    )?;
                    buffered_out = Some(buffer);
                }
            }
            continue;
        }

        let mut command = build_external_command(stage, shell_env);
        let stdout_redirected = files.stdout.is_some();

        // Stdin: file redirect overrides pipe / buffered builtin output.
        if let Some(file) = files.stdin {
            if let Some(mut reader) = prev_stdout.take() {
                let _ = io::copy(&mut reader, &mut io::sink());
            }
            let _ = buffered_out.take();
            command.stdin(Stdio::from(file));
        } else if let Some(buffer) = buffered_out.take() {
            command.stdin(Stdio::piped());
            apply_stdout_for_stage(&mut command, files.stdout, is_last);
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
            prev_stdout = if !stdout_redirected && !is_last {
                child.stdout.take()
            } else {
                None
            };
            children.push(child);
            continue;
        } else if let Some(stdout_pipe) = prev_stdout.take() {
            command.stdin(stdout_pipe);
        }

        apply_stdout_for_stage(&mut command, files.stdout, is_last);

        match command.spawn() {
            Ok(mut child) => {
                prev_stdout = if !stdout_redirected && !is_last {
                    child.stdout.take()
                } else {
                    None
                };
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

    let _ = buffered_out.take();
    let _ = prev_stdout.take();
    let status = wait_children(&mut children)?;
    Ok(CommandResult::Status(status))
}
