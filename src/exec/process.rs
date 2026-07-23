//! Shared child-process helpers used by simple, redirected, and piped execution.

use crate::builtins::{self, BuiltinResult};
use crate::env::ShellEnvironment;
use crate::harden;
use crate::pathfind;

use std::io::{self, Write};
use std::process::{Child, Command, ExitStatus};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

pub(crate) fn exit_status_code(status: ExitStatus) -> u8 {
    if let Some(code) = status.code() {
        return code as u8;
    }

    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        return 128u8.saturating_add(signal as u8);
    }

    1
}

pub(crate) fn build_external_command(argv: &[String], shell_env: &ShellEnvironment) -> Command {
    let program = resolve_program(&argv[0], shell_env);
    let mut command = Command::new(program);
    if argv.len() > 1 {
        command.args(&argv[1..]);
    }
    apply_child_env(&mut command, shell_env);
    crate::jobs::prepare_child_command(&mut command, shell_env);
    command
}

fn resolve_program(name: &str, env: &ShellEnvironment) -> String {
    if name.contains('/') {
        return name.to_owned();
    }
    let path = harden::effective_path(env);
    pathfind::resolve_first(name, &path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_owned())
}

fn apply_child_env(command: &mut Command, shell_env: &ShellEnvironment) {
    command.env_clear();
    let jail = harden::path_jail_enabled(shell_env);
    for (key, value) in shell_env.iter() {
        if jail && key == "PATH" {
            command.env(key, pathfind::sanitize_path(value));
        } else {
            command.env(key, value);
        }
    }
}

pub(crate) fn abandon_children(children: &mut [Child]) {
    for child in children.iter_mut() {
        let _ = child.kill();
    }
}

pub(crate) fn wait_children(children: &mut Vec<Child>) -> io::Result<u8> {
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

/// Run a builtin and map `exit` to a status (pipeline / subshell semantics).
pub(crate) fn run_builtin_status(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match builtins::try_run(argv, shell_env, last_status, stdout, stderr)? {
        Some(BuiltinResult::Status(code) | BuiltinResult::Exit(code)) => Ok(code),
        Some(BuiltinResult::Source(_)) => {
            writeln!(stderr, "source: not available in this context.")?;
            Ok(1)
        }
        Some(BuiltinResult::Repeat { .. }) => {
            writeln!(stderr, "repeat: not available in this context.")?;
            Ok(1)
        }
        None => Ok(0),
    }
}
