//! Expand foreach items and execute the loop body.

use super::body_run::run_body;
use super::control_collect::{collect_body, BlockKind};
use super::line_edit::ReplInput;
use super::ReplIo;
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::foreach::ForEachHeader;
use crate::parse;

use std::io::{self, Write};

/// Collect the body, expand the word list, and run each iteration.
pub(super) fn run_foreach<I: ReplInput, O: Write, E: Write>(
    header: ForEachHeader,
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let Some(body) = collect_body(
        io,
        interactive,
        &shell_env.history,
        &mut shell_env.key_bindings,
        BlockKind::ForEach,
    )?
    else {
        return Ok(CommandResult::Status(1));
    };
    let items = match expand_items(&header.items, shell_env, last_status, io)? {
        Some(items) => items,
        None => return Ok(CommandResult::Status(1)),
    };
    let mut status = last_status;
    for item in items {
        shell_env.set_local(&header.var, item);
        match run_body(&body, io, shell_env, status, argv)? {
            CommandResult::Status(code) => {
                status = code;
                if shell_env.return_requested() {
                    return Ok(CommandResult::Status(code));
                }
            }
            CommandResult::Exit(code) => return Ok(CommandResult::Exit(code)),
            CommandResult::Source(_) => return Ok(CommandResult::Status(1)),
        }
    }
    Ok(CommandResult::Status(status))
}

fn expand_items<I: ReplInput, O: Write, E: Write>(
    raw: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ReplIo<'_, I, O, E>,
) -> io::Result<Option<Vec<String>>> {
    let mut out = Vec::new();
    let words: Vec<&str> = raw.iter().map(String::as_str).collect();
    if let Err(err) = parse::fill_argv(
        &words,
        &mut out,
        shell_env,
        last_status,
        io.stdin,
        io.stderr,
    ) {
        writeln!(io.stderr, "{}", err.message())?;
        return Ok(None);
    }
    Ok(Some(out))
}
