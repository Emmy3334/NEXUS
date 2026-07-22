//! Script file loading for `nexus script.sh` and `source`.

use super::{run_loop, ReplIo};
use crate::env::ShellEnvironment;

use std::collections::VecDeque;
use std::fs;
use std::io::{self, Cursor, Write};
use std::path::Path;

/// Run `path` non-interactively with `argv` as `$0`… and return the exit status.
pub fn run_script(
    path: impl AsRef<Path>,
    argv: Vec<String>,
    stdout: impl Write,
    stderr: impl Write,
) -> io::Result<u8> {
    let path = path.as_ref();
    let contents = read_script(path)?;
    let mut shell_env = ShellEnvironment::capture();
    shell_env.set_argv(argv);
    let mut cursor = Cursor::new(contents);
    run_with_cursor(&mut cursor, stdout, stderr, &mut shell_env)
}

pub(super) fn source_path<O: Write, E: Write>(
    path: &str,
    io: &mut ReplIo<'_, impl super::line_edit::ReplInput, O, E>,
    shell_env: &mut ShellEnvironment,
) -> io::Result<super::LoopEnd> {
    let contents = read_script(Path::new(path))?;
    let mut cursor = Cursor::new(contents);
    let mut nested = ReplIo {
        stdin: &mut cursor,
        stdout: io.stdout,
        stderr: io.stderr,
        input_queue: VecDeque::new(),
    };
    run_loop(&mut nested, false, shell_env)
}

fn run_with_cursor(
    stdin: &mut Cursor<String>,
    mut stdout: impl Write,
    mut stderr: impl Write,
    shell_env: &mut ShellEnvironment,
) -> io::Result<u8> {
    let mut io = ReplIo {
        stdin,
        stdout: &mut stdout,
        stderr: &mut stderr,
        input_queue: VecDeque::new(),
    };
    Ok(match run_loop(&mut io, false, shell_env)? {
        super::LoopEnd::Status(code) | super::LoopEnd::Exit(code) => code,
    })
}

fn read_script(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
        .map_err(|err| io::Error::new(err.kind(), format!("{}: {}", path.display(), err)))
}
