//! Shell builtins for Minishell1: `cd`, `setenv`, `unsetenv`, `env`, `exit`.

use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

/// Outcome of a recognized builtin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use = "builtin status vs shell exit must be handled by the caller"]
pub enum BuiltinResult {
    /// Keep the REPL running with this status.
    Status(u8),
    /// Terminate the shell with this status (`exit`).
    Exit(u8),
}

/// Whether `name` is a Minishell builtin.
#[must_use]
pub fn is_builtin(name: &str) -> bool {
    matches!(name, "cd" | "setenv" | "unsetenv" | "env" | "exit")
}

/// Run a builtin if `argv[0]` matches one; otherwise return `Ok(None)`.
pub fn try_run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<Option<BuiltinResult>> {
    let Some(name) = argv.first().map(String::as_str) else {
        return Ok(None);
    };

    let result = match name {
        "cd" => BuiltinResult::Status(cd(argv, shell_env, stderr)?),
        "setenv" => BuiltinResult::Status(setenv(argv, shell_env, stdout, stderr)?),
        "unsetenv" => BuiltinResult::Status(unsetenv(argv, shell_env, stderr)?),
        "env" => BuiltinResult::Status(env_cmd(argv, shell_env, stdout, stderr)?),
        "exit" => exit_cmd(argv, last_status, stderr)?,
        _ => return Ok(None),
    };

    Ok(Some(result))
}

fn cd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() > 2 {
        writeln!(stderr, "cd: Too many arguments.")?;
        return Ok(1);
    }

    let target = match argv.get(1).map(String::as_str) {
        None | Some("~") => {
            let Some(home) = shell_env.get("HOME") else {
                writeln!(stderr, "cd: No home directory.")?;
                return Ok(1);
            };
            PathBuf::from(home)
        }
        Some("-") => {
            let Some(old) = shell_env.get("OLDPWD") else {
                writeln!(stderr, "cd: OLDPWD not set.")?;
                return Ok(1);
            };
            PathBuf::from(old)
        }
        Some(path) => {
            if let Some(rest) = path.strip_prefix("~/") {
                let Some(home) = shell_env.get("HOME") else {
                    writeln!(stderr, "cd: No home directory.")?;
                    return Ok(1);
                };
                PathBuf::from(home).join(rest)
            } else {
                PathBuf::from(path)
            }
        }
    };

    let previous = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if let Err(err) = process_env::set_current_dir(&target) {
        writeln!(stderr, "cd: {}: {err}", target.display())?;
        return Ok(1);
    }

    let new_pwd = process_env::current_dir().unwrap_or(target);
    shell_env.set("OLDPWD", previous.to_string_lossy().into_owned());
    shell_env.set("PWD", new_pwd.to_string_lossy().into_owned());
    Ok(0)
}

fn setenv(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match argv.len() {
        1 => write_env(shell_env, stdout),
        2 | 3 => {
            let name = &argv[1];
            if !is_valid_env_name(name) {
                writeln!(
                    stderr,
                    "setenv: Variable name must contain alphanumeric characters."
                )?;
                return Ok(1);
            }
            let value = argv.get(2).map(String::as_str).unwrap_or("");
            shell_env.set(name.clone(), value.to_owned());
            Ok(0)
        }
        _ => {
            writeln!(stderr, "setenv: Too many arguments.")?;
            Ok(1)
        }
    }
}

fn unsetenv(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() < 2 {
        writeln!(stderr, "unsetenv: Too few arguments.")?;
        return Ok(1);
    }

    for name in &argv[1..] {
        if name.contains('*') {
            writeln!(stderr, "unsetenv: Wildcard not supported.")?;
            return Ok(1);
        }
        shell_env.unset(name);
    }
    Ok(0)
}

fn env_cmd(
    argv: &[String],
    shell_env: &ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() != 1 {
        writeln!(stderr, "env: Too many arguments.")?;
        return Ok(1);
    }
    write_env(shell_env, stdout)
}

fn write_env(shell_env: &ShellEnvironment, stdout: &mut impl Write) -> io::Result<u8> {
    for (name, value) in shell_env.iter() {
        writeln!(stdout, "{name}={value}")?;
    }
    Ok(0)
}

fn exit_cmd(
    argv: &[String],
    last_status: u8,
    stderr: &mut impl Write,
) -> io::Result<BuiltinResult> {
    match argv.len() {
        1 => Ok(BuiltinResult::Exit(last_status)),
        2 => match parse_exit_status(&argv[1]) {
            Some(code) => Ok(BuiltinResult::Exit(code)),
            None => {
                writeln!(stderr, "exit: Expression Syntax.")?;
                Ok(BuiltinResult::Status(1))
            }
        },
        _ => {
            writeln!(stderr, "exit: Expression Syntax.")?;
            Ok(BuiltinResult::Status(1))
        }
    }
}

fn parse_exit_status(text: &str) -> Option<u8> {
    text.parse::<i32>().ok().map(|n| n as u8)
}

fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;

    /// Serialize tests that mutate the process working directory.
    static CWD_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn empty_env() -> ShellEnvironment {
        ShellEnvironment::from_map(BTreeMap::new())
    }

    fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
        CWD_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn env_prints_sorted_assignments() {
        let mut shell_env = empty_env();
        shell_env.set("B", "2");
        shell_env.set("A", "1");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = env_cmd(&["env".into()], &shell_env, &mut stdout, &mut stderr).unwrap();
        assert_eq!(code, 0);
        assert_eq!(String::from_utf8(stdout).unwrap(), "A=1\nB=2\n");
        assert!(stderr.is_empty());
    }

    #[test]
    fn env_rejects_arguments() {
        let shell_env = empty_env();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = env_cmd(
            &["env".into(), "EXTRA".into()],
            &shell_env,
            &mut stdout,
            &mut stderr,
        )
        .unwrap();
        assert_eq!(code, 1);
        assert!(String::from_utf8(stderr).unwrap().contains("Too many"));
    }

    #[test]
    fn setenv_and_unsetenv() {
        let mut shell_env = empty_env();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            setenv(
                &["setenv".into(), "FOO".into(), "bar".into()],
                &mut shell_env,
                &mut stdout,
                &mut stderr
            )
            .unwrap(),
            0
        );
        assert_eq!(shell_env.get("FOO"), Some("bar"));
        assert_eq!(
            unsetenv(
                &["unsetenv".into(), "FOO".into()],
                &mut shell_env,
                &mut stderr
            )
            .unwrap(),
            0
        );
        assert!(!shell_env.contains("FOO"));
    }

    #[test]
    fn unsetenv_rejects_wildcard() {
        let mut shell_env = empty_env();
        let mut stderr = Vec::new();
        let code = unsetenv(
            &["unsetenv".into(), "FOO*".into()],
            &mut shell_env,
            &mut stderr,
        )
        .unwrap();
        assert_eq!(code, 1);
        assert!(String::from_utf8(stderr).unwrap().contains("Wildcard"));
    }

    #[test]
    fn exit_with_code() {
        let mut stderr = Vec::new();
        let result = exit_cmd(&["exit".into(), "42".into()], 0, &mut stderr).unwrap();
        assert_eq!(result, BuiltinResult::Exit(42));
    }

    #[test]
    fn exit_without_arg_uses_last_status() {
        let mut stderr = Vec::new();
        let result = exit_cmd(&["exit".into()], 7, &mut stderr).unwrap();
        assert_eq!(result, BuiltinResult::Exit(7));
    }

    #[test]
    fn cd_changes_directory() {
        let _cwd_guard = lock_cwd();
        let start = process_env::current_dir().unwrap();
        let scratch = process_env::temp_dir().join(format!("nexus-cd-test-{}", std::process::id()));
        let nested = scratch.join("nested");
        fs::create_dir_all(&nested).unwrap();

        let mut shell_env = empty_env();
        shell_env.set("HOME", scratch.to_string_lossy());
        let mut stderr = Vec::new();

        let code = cd(
            &["cd".into(), nested.to_string_lossy().into_owned()],
            &mut shell_env,
            &mut stderr,
        )
        .unwrap();
        assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));
        assert_eq!(
            process_env::current_dir().unwrap().canonicalize().unwrap(),
            nested.canonicalize().unwrap()
        );
        assert!(Path::new(shell_env.get("PWD").unwrap()).exists());

        let code = cd(
            &["cd".into(), "~/nested".into()],
            &mut shell_env,
            &mut stderr,
        )
        .unwrap();
        assert_eq!(code, 0, "stderr={}", String::from_utf8_lossy(&stderr));

        process_env::set_current_dir(&start).unwrap();
        let _ = fs::remove_dir_all(&scratch);
    }
}
