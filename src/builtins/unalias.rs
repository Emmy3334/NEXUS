//! `unalias` builtin — remove command aliases.

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn unalias(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() < 2 {
        writeln!(stderr, "unalias: Too few arguments.")?;
        return Ok(1);
    }
    for name in &argv[1..] {
        if name.contains('*') {
            writeln!(stderr, "unalias: Wildcard not supported.")?;
            return Ok(1);
        }
        shell_env.alias_unset(name);
    }
    Ok(0)
}
