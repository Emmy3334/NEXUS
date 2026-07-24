//! `compdump` — write the completion registry to the dump file.

use super::path;
use crate::env::save_dump;
use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &ShellEnvironment,
    _stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let dump_path = match path::parse_dump_path(argv, shell_env, stderr)? {
        Some(p) => p,
        None => return Ok(1),
    };
    match save_dump(&dump_path, &shell_env.comp_registry) {
        Ok(()) => Ok(0),
        Err(err) => {
            writeln!(stderr, "compdump: {err}")?;
            Ok(1)
        }
    }
}
