//! Run special aliases (`precmd`, `cwdcmd`) when defined.

use crate::alias;
use crate::env::ShellEnvironment;
use crate::exec::{self, CommandResult};

use std::cell::Cell;
use std::io::{self, BufRead, Write};

thread_local! {
    static IN_HOOK: Cell<bool> = const { Cell::new(false) };
}

/// Run the `precmd` alias just before an interactive prompt, if set.
pub fn run_precmd(
    env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<()> {
    run_named("precmd", env, last_status, stdin, stdout, stderr)
}

/// Run the `cwdcmd` alias after a successful directory change, if set.
pub fn run_cwdcmd(
    env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<()> {
    run_named("cwdcmd", env, last_status, stdin, stdout, stderr)
}

fn run_named(
    name: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<()> {
    if env.alias_get(name).is_none() {
        return Ok(());
    }
    if IN_HOOK.get() {
        return Ok(());
    }
    IN_HOOK.set(true);
    let result = run_alias_body(name, env, last_status, stdin, stdout, stderr);
    IN_HOOK.set(false);
    result
}

fn run_alias_body(
    name: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<()> {
    let mut argv = vec![name.to_owned()];
    if let Err(err) = alias::apply_aliases(&mut argv, env, last_status, stdin, stderr) {
        writeln!(stderr, "{}", err.message())?;
        return Ok(());
    }
    if argv.is_empty() {
        return Ok(());
    }
    match exec::execute_command(&argv, env, last_status, stdout, stderr)? {
        CommandResult::Status(_) | CommandResult::Exit(_) => Ok(()),
        CommandResult::Source(_) => {
            writeln!(stderr, "{name}: source not available here.")?;
            Ok(())
        }
    }
}
