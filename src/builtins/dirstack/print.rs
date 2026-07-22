//! Print the directory stack (`dirs` / `pushd` / `popd` output).

use super::args::PrintFlags;
use crate::env::ShellEnvironment;

use std::io::{self, Write};
use std::path::Path;

pub(super) fn print_stack(
    shell_env: &ShellEnvironment,
    flags: PrintFlags,
    stdout: &mut impl Write,
) -> io::Result<u8> {
    let home = shell_env
        .lookup("home")
        .or_else(|| shell_env.lookup("HOME"));
    if flags.verbose {
        for (i, path) in shell_env.dir_stack.iter().enumerate() {
            writeln!(stdout, "{i}\t{}", format_path(path, home, flags.long))?;
        }
        return Ok(0);
    }
    write_horizontal(shell_env, home, flags, stdout)?;
    writeln!(stdout)?;
    Ok(0)
}

fn write_horizontal(
    shell_env: &ShellEnvironment,
    home: Option<&str>,
    flags: PrintFlags,
    stdout: &mut impl Write,
) -> io::Result<()> {
    let width = columns();
    let mut col = 0_usize;
    for (i, path) in shell_env.dir_stack.iter().enumerate() {
        let text = format_path(path, home, flags.long);
        if i > 0 {
            if flags.wrap && col + 1 + text.len() > width && col > 0 {
                writeln!(stdout)?;
                col = 0;
            } else {
                write!(stdout, " ")?;
                col += 1;
            }
        }
        write!(stdout, "{text}")?;
        col += text.len();
    }
    Ok(())
}

fn format_path(path: &Path, home: Option<&str>, long: bool) -> String {
    let raw = path.to_string_lossy();
    if long {
        return raw.into_owned();
    }
    let Some(home) = home else {
        return raw.into_owned();
    };
    if raw.as_ref() == home {
        return "~".into();
    }
    let prefix = format!("{home}/");
    match raw.strip_prefix(&prefix) {
        Some(rest) => format!("~/{rest}"),
        None => raw.into_owned(),
    }
}

fn columns() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
        .max(1)
}
