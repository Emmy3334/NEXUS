//! Zsh-style completion registry builtins (`compdef`, `compinit`, `compdump`).

mod compdef;
mod compdump;
mod compinit;
mod path;

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    name: &str,
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match name {
        "compdef" => compdef::run(argv, shell_env, stderr),
        "compinit" => compinit::run(argv, shell_env, stderr),
        "compdump" => compdump::run(argv, shell_env, stdout, stderr),
        _ => unreachable!(),
    }
}
