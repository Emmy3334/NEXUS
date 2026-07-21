//! Command execution: builtins first, then external programs.

use crate::builtins::{self, BuiltinResult};
use crate::env::ShellEnvironment;

use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Command, ExitStatus};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

/// Outcome of running one simple command.
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
}
