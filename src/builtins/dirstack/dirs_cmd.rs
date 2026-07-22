//! `dirs` — print the directory stack.

use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

pub(crate) fn dirs_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() > 1 {
        writeln!(stderr, "dirs: Too many arguments.")?;
        return Ok(1);
    }
    let cwd = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    shell_env.dir_stack.ensure_seeded(cwd);
    for (i, path) in shell_env.dir_stack.iter().enumerate() {
        if i > 0 {
            write!(stdout, " ")?;
        }
        write!(stdout, "{}", path.display())?;
    }
    writeln!(stdout)?;
    Ok(0)
}
