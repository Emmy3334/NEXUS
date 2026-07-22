//! Evaluate `if` conditions and run the matching branch body.

use super::foreach_run::run_body;
use super::if_collect::{collect_if, IfPlan};
use super::line_edit::ReplInput;
use super::ReplIo;
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::if_block::IfHeader;
use crate::while_loop;

use std::io::{self, Write};

/// Collect branches and run the first true one (or else).
pub(super) fn run_if<I: ReplInput, O: Write, E: Write>(
    header: IfHeader,
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let Some(plan) = collect_if(io, interactive, &shell_env.history)? else {
        return Ok(CommandResult::Status(1));
    };
    run_plan(header, plan, io, shell_env, last_status, argv)
}

fn run_plan<I: ReplInput, O: Write, E: Write>(
    header: IfHeader,
    plan: IfPlan,
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    match eval_branch(&header.expr, shell_env, last_status, io) {
        None => return Ok(CommandResult::Status(1)),
        Some(true) => return run_body(&plan.then_body, io, shell_env, last_status, argv),
        Some(false) => {}
    }
    for (branch, body) in &plan.else_ifs {
        match eval_branch(&branch.expr, shell_env, last_status, io) {
            None => return Ok(CommandResult::Status(1)),
            Some(true) => return run_body(body, io, shell_env, last_status, argv),
            Some(false) => {}
        }
    }
    if let Some(body) = &plan.else_body {
        return run_body(body, io, shell_env, last_status, argv);
    }
    Ok(CommandResult::Status(last_status))
}

fn eval_branch<I: ReplInput, O: Write, E: Write>(
    expr: &[String],
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    io: &mut ReplIo<'_, I, O, E>,
) -> Option<bool> {
    while_loop::eval_condition(expr, shell_env, last_status, io.stdin, io.stderr)
}
