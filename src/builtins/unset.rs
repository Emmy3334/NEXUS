//! `unset` builtin — remove shell-local variables.

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn unset(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() < 2 {
        writeln!(stderr, "unset: Too few arguments.")?;
        return Ok(1);
    }

    for name in &argv[1..] {
        if name.contains('*') {
            writeln!(stderr, "unset: Wildcard not supported.")?;
            return Ok(1);
        }
        shell_env.unset_local(name);
    }
    Ok(0)
}
