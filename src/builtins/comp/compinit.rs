//! `compinit` — load completion dump into the registry.

use super::path;
use crate::env::load_dump;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let dump_path = match path::parse_init_path(argv, shell_env, stderr)? {
        Some(p) => p,
        None => return Ok(1),
    };
    match load_dump(&dump_path) {
        Ok(Some(reg)) => {
            shell_env.comp_registry = reg;
            Ok(0)
        }
        Ok(None) => Ok(0),
        Err(err) => {
            writeln!(stderr, "compinit: {err}")?;
            Ok(1)
        }
    }
}
