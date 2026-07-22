//! `popd` — pop directory stack and cd.

use crate::builtins::cd;
use crate::env::ShellEnvironment;

use std::env as process_env;
use std::io::{self, Write};
use std::path::PathBuf;

pub(crate) fn popd_cmd(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() > 1 {
        writeln!(stderr, "popd: Too many arguments.")?;
        return Ok(1);
    }
    let cwd = process_env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    shell_env.dir_stack.ensure_seeded(cwd);
    if shell_env.dir_stack.len() < 2 {
        writeln!(stderr, "popd: Directory stack empty.")?;
        return Ok(1);
    }
    let _ = shell_env.dir_stack.pop();
    let Some(next) = shell_env.dir_stack.iter().next().cloned() else {
        writeln!(stderr, "popd: Directory stack empty.")?;
        return Ok(1);
    };
    let code = cd::change_directory(&next, shell_env, last_status, stdout, stderr)?;
    if code != 0 {
        return Ok(code);
    }
    for (i, path) in shell_env.dir_stack.iter().enumerate() {
        if i > 0 {
            write!(stdout, " ")?;
        }
        write!(stdout, "{}", path.display())?;
    }
    writeln!(stdout)?;
    Ok(0)
}
