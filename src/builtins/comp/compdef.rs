//! `compdef cmd word [word…]` / `compdef -d cmd`.

use crate::env::ShellEnvironment;

use std::io::{self, Write};

pub(super) fn run(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() < 2 {
        return usage(stderr);
    }
    if argv[1] == "-d" {
        return delete(argv, shell_env, stderr);
    }
    if argv.len() < 3 {
        return usage(stderr);
    }
    let cmd = argv[1].as_str();
    shell_env.comp_registry.register(cmd, argv[2..].to_vec());
    Ok(0)
}

fn delete(
    argv: &[String],
    shell_env: &mut ShellEnvironment,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if argv.len() != 3 {
        return usage(stderr);
    }
    shell_env.comp_registry.delete(&argv[2]);
    Ok(0)
}

fn usage(stderr: &mut impl Write) -> io::Result<u8> {
    writeln!(
        stderr,
        "compdef: usage: compdef cmd word [word…] | compdef -d cmd"
    )?;
    Ok(1)
}
