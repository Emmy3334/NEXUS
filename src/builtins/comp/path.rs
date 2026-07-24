//! Shared dump-path parsing for `compinit` / `compdump`.

use crate::env::resolve_dump_path;
use crate::env::ShellEnvironment;

use std::io::{self, Write};
use std::path::PathBuf;

pub(super) fn parse_init_path(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<Option<PathBuf>> {
    match argv.len() {
        1 => resolve_or_err(shell_env, stderr),
        3 if argv[1] == "-d" => {
            shell_env.set_local("compdump", argv[2].clone());
            Ok(Some(PathBuf::from(&argv[2])))
        }
        _ => {
            usage_init(stderr)?;
            Ok(None)
        }
    }
}

pub(super) fn parse_dump_path(
    argv: &[String],
    shell_env: &ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<Option<PathBuf>> {
    match argv.len() {
        1 => resolve_or_err(shell_env, stderr),
        3 if argv[1] == "-d" => Ok(Some(PathBuf::from(&argv[2]))),
        _ => {
            usage_dump(stderr)?;
            Ok(None)
        }
    }
}

fn resolve_or_err(
    shell_env: &ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<Option<PathBuf>> {
    match resolve_dump_path(shell_env) {
        Some(p) => Ok(Some(p)),
        None => {
            writeln!(stderr, "compdump: HOME not set")?;
            Ok(None)
        }
    }
}

fn usage_init(stderr: &mut impl Write) -> io::Result<()> {
    writeln!(stderr, "compinit: usage: compinit [-d path]")?;
    Ok(())
}

fn usage_dump(stderr: &mut impl Write) -> io::Result<()> {
    writeln!(stderr, "compdump: usage: compdump [-d path]")?;
    Ok(())
}
