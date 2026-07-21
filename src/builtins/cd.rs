//! `cd` builtin — change working directory and update `PWD` / `OLDPWD`.

use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub(super) fn cd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() > 2 {
        writeln!(stderr, "cd: Too many arguments.")?;
        return Ok(1);
    }

    let target = match resolve_target(argv, shell_env, stderr)? {
        Ok(target) => target,
        Err(code) => return Ok(code),
    };

    change_directory(&target, shell_env, stderr)
}

/// Resolve the requested argument (`~`, `-`, `~/…`, or a plain path) into a
/// concrete target directory.
fn resolve_target(
    argv: &[String],
    shell_env: &ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<Result<PathBuf, u8>> {
    match argv.get(1).map(String::as_str) {
        None | Some("~") => match shell_env.get("HOME") {
            Some(home) => Ok(Ok(PathBuf::from(home))),
            None => {
                writeln!(stderr, "cd: No home directory.")?;
                Ok(Err(1))
            }
        },
        Some("-") => match shell_env.get("OLDPWD") {
            Some(old) => Ok(Ok(PathBuf::from(old))),
            None => {
                writeln!(stderr, "cd: OLDPWD not set.")?;
                Ok(Err(1))
            }
        },
        Some(path) => match path.strip_prefix("~/") {
            Some(rest) => match shell_env.get("HOME") {
                Some(home) => Ok(Ok(PathBuf::from(home).join(rest))),
                None => {
                    writeln!(stderr, "cd: No home directory.")?;
                    Ok(Err(1))
                }
            },
            None => Ok(Ok(PathBuf::from(path))),
        },
    }
}

/// Change into `target`, then refresh `PWD` / `OLDPWD` / the `cwd` local.
fn change_directory(
    target: &Path,
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let previous = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if let Err(err) = process_env::set_current_dir(target) {
        writeln!(stderr, "cd: {}: {err}", target.display())?;
        return Ok(1);
    }

    let new_pwd = process_env::current_dir().unwrap_or_else(|_| target.to_path_buf());
    let pwd = new_pwd.to_string_lossy().into_owned();
    shell_env.set("OLDPWD", previous.to_string_lossy().into_owned());
    shell_env.set("PWD", pwd.clone());
    shell_env.set_cwd(pwd);
    Ok(0)
}
