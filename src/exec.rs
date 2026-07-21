//! Command execution: builtins first, then external programs.
//!
//! Walks a [`CommandList`]: `;` runs pipelines in order; `|` connects
//! stages with OS pipes. Builtins in a multi-stage pipeline use subshell
//! semantics (cloned env; `exit` does not kill the parent shell).

use crate::builtins::{self, BuiltinResult};
use crate::env::ShellEnvironment;
use crate::parse::{self, CommandList, Pipeline};

use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Child, Command, ExitStatus, Stdio};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

/// Outcome of running one simple command or a list/pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "shell exit vs continue must be handled by the REPL"]
pub enum CommandResult {
    /// Keep the REPL running with this status.
    Status(u8),
    /// Terminate the shell with this status (`exit` builtin).
    Exit(u8),
}

impl From<BuiltinResult> for CommandResult {
    fn from(result: BuiltinResult) -> Self {
        match result {
            BuiltinResult::Status(code) => Self::Status(code),
            BuiltinResult::Exit(code) => Self::Exit(code),
        }
    }
}

/// Run a command list: each `;`-separated pipeline in order.
///
/// `exit` in a single-command pipeline stops the shell. Reuses `argv`
/// across simple (non-pipe) commands.
pub fn execute_list(
    list: &CommandList<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    mut last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    for pipeline in &list.pipelines {
        match execute_pipeline(pipeline, argv, shell_env, last_status, stdout, stderr)? {
            CommandResult::Status(code) => last_status = code,
            CommandResult::Exit(code) => return Ok(CommandResult::Exit(code)),
        }
    }

    Ok(CommandResult::Status(last_status))
}

fn execute_pipeline(
    pipeline: &Pipeline<'_>,
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    match pipeline.commands.as_slice() {
        [] => Ok(CommandResult::Status(last_status)),
        [simple] => {
            parse::fill_argv(&simple.argv, argv);
            if argv.is_empty() {
                return Ok(CommandResult::Status(last_status));
            }
            execute_command(argv, shell_env, last_status, stdout, stderr)
        }
        _ => execute_piped_stages(pipeline, shell_env, last_status, stdout, stderr),
    }
}

/// Multi-stage `|` pipeline. Status is the last stage’s status.
fn execute_piped_stages(
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

/// Run a builtin and map `exit` to a status (pipeline / subshell semantics).
fn run_builtin_status(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match builtins::try_run(argv, shell_env, last_status, stdout, stderr)? {
        Some(BuiltinResult::Status(code) | BuiltinResult::Exit(code)) => Ok(code),
        None => Ok(0),
    }
}

fn build_external_command(argv: &[String], shell_env: &ShellEnvironment) -> Command {
    let mut command = Command::new(&argv[0]);
    if argv.len() > 1 {
        command.args(&argv[1..]);
    }
    command.env_clear().envs(shell_env.iter());
    command
}

fn abandon_children(children: &mut [Child]) {
    for child in children.iter_mut() {
        let _ = child.kill();
    }
}

fn wait_children(children: &mut Vec<Child>) -> io::Result<u8> {
    let count = children.len();
    let mut last_status = 0u8;
    for (index, mut child) in children.drain(..).enumerate() {
        let status = child.wait()?;
        if index + 1 == count {
            last_status = exit_status_code(status);
        }
    }
    Ok(last_status)
}

/// Dispatch a simple command through builtins or an external spawn.
pub fn execute_command(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<CommandResult> {
    if let Some(result) = builtins::try_run(argv, shell_env, last_status, stdout, stderr)? {
        return Ok(result.into());
    }

    Ok(CommandResult::Status(execute_external(
        argv, shell_env, stderr,
    )?))
}

/// Run an external program with the shell's environment copy.
///
/// - Resolves via absolute/relative path or `PATH`.
/// - On “not found”, writes `{name}: Command not found.` to `stderr` and
///   returns `127`.
pub fn execute_external<S: AsRef<OsStr>>(
    argv: &[S],
    shell_env: &ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };

    match Command::new(program)
        .args(args)
        .env_clear()
        .envs(shell_env.iter())
        .status()
    {
        Ok(status) => Ok(exit_status_code(status)),
        Err(err) => {
            let name = program.as_ref().to_string_lossy();
            Ok(report_spawn_failure(&name, &err, stderr)?)
        }
    }
}

fn report_spawn_failure(program: &str, err: &io::Error, stderr: &mut impl Write) -> io::Result<u8> {
    match err.kind() {
        io::ErrorKind::NotFound => {
            writeln!(stderr, "{program}: Command not found.")?;
            Ok(127)
        }
        io::ErrorKind::PermissionDenied => {
            writeln!(stderr, "{program}: Permission denied.")?;
            Ok(126)
        }
        _ => {
            writeln!(stderr, "{program}: {err}")?;
            Ok(1)
        }
    }
}

fn exit_status_code(status: ExitStatus) -> u8 {
    if let Some(code) = status.code() {
        return code as u8;
    }

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        return 128u8.saturating_add(signal as u8);
    }

    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn test_env() -> ShellEnvironment {
        // Keep PATH so `true` / `false` resolve in tests.
        let path = std::env::var("PATH").unwrap_or_default();
        let mut map = BTreeMap::new();
        map.insert("PATH".into(), path);
        ShellEnvironment::from_map(map)
    }

    fn parse_list(source: &str) -> CommandList<'_> {
        let mut tokens = Vec::new();
        crate::lex::tokenize_into(source, &mut tokens);
        crate::parse::parse_line(source, &tokens)
            .expect("parse ok")
            .expect("non-empty list")
    }

    #[test]
    fn true_exits_zero() {
        let env = test_env();
        let mut stderr = Vec::new();
        let code = execute_external(&["true"], &env, &mut stderr).unwrap();
        assert_eq!(code, 0);
        assert!(stderr.is_empty());
    }

    #[test]
    fn false_exits_one() {
        let env = test_env();
        let mut stderr = Vec::new();
        let code = execute_external(&["false"], &env, &mut stderr).unwrap();
        assert_eq!(code, 1);
        assert!(stderr.is_empty());
    }

    #[test]
    fn missing_command_is_127_with_message() {
        let env = test_env();
        let mut stderr = Vec::new();
        let code = execute_external(&["nexus_no_such_command_42"], &env, &mut stderr).unwrap();
        assert_eq!(code, 127);
        let message = String::from_utf8(stderr).unwrap();
        assert!(message.contains("Command not found"));
        assert!(message.contains("nexus_no_such_command_42"));
    }

    #[test]
    fn absolute_true_path() {
        let true_path = ["/usr/bin/true", "/bin/true"]
            .into_iter()
            .find(|path| Path::new(path).exists())
            .expect("system true binary");

        let env = test_env();
        let mut stderr = Vec::new();
        let code = execute_external(&[true_path], &env, &mut stderr).unwrap();
        assert_eq!(code, 0);
        assert!(stderr.is_empty());
    }

    #[test]
    fn exit_builtin_stops_shell() {
        let mut env = test_env();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_command(
            &["exit".into(), "3".into()],
            &mut env,
            0,
            &mut stdout,
            &mut stderr,
        )
        .unwrap();
        assert_eq!(result, CommandResult::Exit(3));
    }

    #[test]
    fn external_inherits_shell_path_only_env() {
        let mut map = BTreeMap::new();
        map.insert("PATH".into(), "/nonexistent".into());
        let env = ShellEnvironment::from_map(map);
        let mut stderr = Vec::new();
        let code = execute_external(&["true"], &env, &mut stderr).unwrap();
        assert_eq!(code, 127);
        assert!(String::from_utf8(stderr)
            .unwrap()
            .contains("Command not found"));
    }

    #[test]
    fn semicolon_list_runs_in_sequence() {
        let source = "false ; true";
        let list = parse_list(source);
        let mut env = test_env();
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(0));
        assert!(stderr.is_empty());
    }

    #[test]
    fn semicolon_list_keeps_last_status() {
        let source = "true ; false";
        let list = parse_list(source);
        let mut env = test_env();
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(1));
    }

    #[test]
    fn exit_in_list_stops_remaining_commands() {
        let source = "exit 7 ; false";
        let list = parse_list(source);
        let mut env = test_env();
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Exit(7));
    }

    #[test]
    fn pipe_uses_last_command_status() {
        let source = "true | false";
        let list = parse_list(source);
        let mut env = test_env();
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(1));

        let source = "false | true";
        let list = parse_list(source);
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(0));
    }

    #[test]
    fn multipipe_status_is_last_stage() {
        let source = "true | true | false";
        let list = parse_list(source);
        let mut env = test_env();
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(1));
    }

    #[test]
    fn exit_in_pipeline_does_not_kill_shell() {
        let source = "exit 9 | true";
        let list = parse_list(source);
        let mut env = test_env();
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(0));
    }

    #[test]
    fn builtin_env_can_feed_pipe() {
        let source = "env | true";
        let list = parse_list(source);
        let mut env = test_env();
        env.set("NEXUS_PIPE_TEST", "1");
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(0));
        assert!(stderr.is_empty());
    }

    #[test]
    fn semicolon_then_pipe() {
        let source = "false ; true | false";
        let list = parse_list(source);
        let mut env = test_env();
        let mut argv = Vec::new();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = execute_list(&list, &mut argv, &mut env, 0, &mut stdout, &mut stderr).unwrap();
        assert_eq!(result, CommandResult::Status(1));
    }
}
