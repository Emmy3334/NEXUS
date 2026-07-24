//! Run a `( list )` subshell with optional group redirects.
//!
//! Uses a cloned environment and restores the process cwd afterward so
//! `cd` inside the group does not leak to the parent (no OS fork required).

mod stdio;

use super::cwd;
use super::io::ExecIo;
use super::redirect::{self, HeredocState};
use super::CommandResult;
use crate::env::ShellEnvironment;
use crate::parse::{CommandList, Redirect};

use std::io::{self, BufRead, Write};

/// Execute an isolated command list, restoring cwd when finished.
pub(super) fn run_subshell<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    argv: &mut Vec<String>,
    shell_env: &ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    let saved_cwd = cwd::save();
    let mut sub_env = shell_env.clone_for_capture();
    let result = run_isolated(
        list,
        redirects,
        argv,
        &mut sub_env,
        last_status,
        heredocs,
        io,
    );
    cwd::restore(saved_cwd, io.stderr);
    Ok(match result? {
        CommandResult::Exit(code) | CommandResult::Status(code) => CommandResult::Status(code),
        CommandResult::Source(_) => CommandResult::Status(1),
    })
}

fn run_isolated<I: BufRead, O: Write, E: Write>(
    list: &CommandList<'_>,
    redirects: &[Redirect<'_>],
    argv: &mut Vec<String>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    heredocs: &mut HeredocState,
    io: &mut ExecIo<'_, I, O, E>,
) -> io::Result<CommandResult> {
    if redirects.is_empty() {
        return super::list::execute_list_with(list, argv, shell_env, last_status, heredocs, io);
    }
    let files = match redirect::open_redirect_files(
        redirects,
        heredocs,
        shell_env,
        last_status,
        io.stdin,
        io.stderr,
    )? {
        Ok(files) => files,
        Err(code) => return Ok(CommandResult::Status(code)),
    };
    stdio::apply_group_stdio(list, argv, shell_env, last_status, heredocs, files, io)
}
