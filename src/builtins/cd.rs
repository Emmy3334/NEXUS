//! `cd` builtin — change working directory and update `PWD` / `OLDPWD`.

use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

pub(super) fn cd(
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
    let pwd = new_pwd.to_string_lossy().into_owned();
    shell_env.set("OLDPWD", previous.to_string_lossy().into_owned());
    shell_env.set("PWD", pwd.clone());
    shell_env.set_cwd(pwd);
    Ok(0)
}
