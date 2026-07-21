//! `|` pipeline execution — OS pipes between simple-command stages.
//!
//! Builtins in a multi-stage pipeline use subshell semantics (cloned env;
//! `exit` does not kill the parent shell). Status is the last stage’s status.
//! File / heredoc redirects on a stage override the pipe on that fd.

use super::redirect::{apply_stdout_for_stage, open_redirect_files, HeredocState, StdinSource};
use super::{
    abandon_children, build_external_command, report_spawn_failure, run_builtin_status,
    wait_children, CommandResult,
};
use crate::builtins;
use crate::env::ShellEnvironment;
use crate::parse::{Pipeline, Redirect};

use std::io::{self, Write};
use std::process::{Child, ChildStdout, Stdio};

/// What to do after a pipeline stage fails to spawn.
enum AfterSpawnFail {
    /// Skip this stage; later stages still run (no pipefail).
    Continue,
    /// This was the last stage — pipeline status is the spawn failure code.
    Done(CommandResult),
}

/// Report spawn failure, drain unused pipe input, then continue or finish.
fn after_spawn_failure(
    name: &str,
    err: &io::Error,
    stderr: &mut impl Write,
    is_last: bool,
    prev_stdout: &mut Option<ChildStdout>,
    buffered_out: &mut Option<Vec<u8>>,
    children: &mut Vec<Child>,
) -> io::Result<AfterSpawnFail> {
    let code = report_spawn_failure(name, err, stderr)?;
    // Prior writers may still be on the pipe — drain so they can exit.
    if let Some(mut reader) = prev_stdout.take() {
        let _ = io::copy(&mut reader, &mut io::sink());
    }
    let _ = buffered_out.take();
    if is_last {
        let _ = wait_children(children)?;
        return Ok(AfterSpawnFail::Done(CommandResult::Status(code)));
    }
    Ok(AfterSpawnFail::Continue)
}

/// Multi-stage `|` pipeline. Status is the last stage’s status.
pub(super) fn execute_piped_stages(
    pipeline: &Pipeline<'_>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    let mut stages: Vec<(Vec<String>, &[Redirect<'_>])> =
        Vec::with_capacity(pipeline.commands.len());
    for command in &pipeline.commands {
        let mut argv = Vec::with_capacity(command.argv.len());
        for word in &command.argv {
            match crate::expand::expand_word_for_exec(word, shell_env, last_status) {
                Ok(expanded) => argv.extend(crate::glob::expand_globs(&expanded)),
                Err(err) => {
                    writeln!(stderr, "{}", err.message())?;
                    return Ok(CommandResult::Status(1));
                }
            }
        }
        stages.push((argv, command.redirects.as_slice()));
    }

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

        let files = match open_redirect_files(redirects, heredocs, shell_env, last_status, stderr)?
        {
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
            // Builtins don't consume stdin yet; drop any stdin redirect.
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

        // Stdin: file / heredoc redirect overrides pipe / buffered builtin output.
        let stdin_bytes = match files.stdin {
            Some(StdinSource::File(file)) => {
                if let Some(mut reader) = prev_stdout.take() {
                    let _ = io::copy(&mut reader, &mut io::sink());
                }
                let _ = buffered_out.take();
                command.stdin(Stdio::from(file));
                None
            }
            Some(StdinSource::Bytes(bytes)) => {
                if let Some(mut reader) = prev_stdout.take() {
                    let _ = io::copy(&mut reader, &mut io::sink());
                }
                let _ = buffered_out.take();
                command.stdin(Stdio::piped());
                Some(bytes)
            }
            None => {
                if let Some(buffer) = buffered_out.take() {
                    command.stdin(Stdio::piped());
                    apply_stdout_for_stage(&mut command, files.stdout, is_last);
                    let mut child = match command.spawn() {
                        Ok(child) => child,
                        Err(err) => {
                            match after_spawn_failure(
                                name,
                                &err,
                                stderr,
                                is_last,
                                &mut prev_stdout,
                                &mut buffered_out,
                                &mut children,
                            )? {
                                AfterSpawnFail::Continue => continue,
                                AfterSpawnFail::Done(result) => return Ok(result),
                            }
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
                }
                if let Some(stdout_pipe) = prev_stdout.take() {
                    command.stdin(stdout_pipe);
                }
                None
            }
        };

        apply_stdout_for_stage(&mut command, files.stdout, is_last);

        match command.spawn() {
            Ok(mut child) => {
                if let Some(bytes) = stdin_bytes {
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(&bytes)?;
                    }
                }
                prev_stdout = if !stdout_redirected && !is_last {
                    child.stdout.take()
                } else {
                    None
                };
                children.push(child);
            }
            Err(err) => {
                match after_spawn_failure(
                    name,
                    &err,
                    stderr,
                    is_last,
                    &mut prev_stdout,
                    &mut buffered_out,
                    &mut children,
                )? {
                    AfterSpawnFail::Continue => continue,
                    AfterSpawnFail::Done(result) => return Ok(result),
                }
            }
        }
    }

    let _ = buffered_out.take();
    let _ = prev_stdout.take();
    let status = wait_children(&mut children)?;
    Ok(CommandResult::Status(status))
}
