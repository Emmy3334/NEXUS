//! External command execution (Minishell1: PATH / direct path, no builtins yet).

use std::ffi::OsStr;
use std::io::{self, Write};
use std::process::{Command, ExitStatus};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

/// Run an external program and return its shell-style exit status (0–255).
///
/// - Resolves via absolute/relative path or `PATH` (handled by the OS/`Command`).
/// - On “not found”, writes `{name}: Command not found.` to `stderr` and
///   returns `127`.
pub fn execute_external<S: AsRef<OsStr>>(argv: &[S], stderr: &mut impl Write) -> io::Result<u8> {
    let Some((program, args)) = argv.split_first() else {
        return Ok(0);
    };

    match Command::new(program).args(args).status() {
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
    use std::path::Path;

    #[test]
    fn true_exits_zero() {
        let mut stderr = Vec::new();
        let code = execute_external(&["true"], &mut stderr).unwrap();
        assert_eq!(code, 0);
        assert!(stderr.is_empty());
    }

    #[test]
    fn false_exits_one() {
        let mut stderr = Vec::new();
        let code = execute_external(&["false"], &mut stderr).unwrap();
        assert_eq!(code, 1);
        assert!(stderr.is_empty());
    }

    #[test]
    fn missing_command_is_127_with_message() {
        let mut stderr = Vec::new();
        let code = execute_external(&["nexus_no_such_command_42"], &mut stderr).unwrap();
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

        let mut stderr = Vec::new();
        let code = execute_external(&[true_path], &mut stderr).unwrap();
        assert_eq!(code, 0);
        assert!(stderr.is_empty());
    }
}
