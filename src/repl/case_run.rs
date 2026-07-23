//! Expand the case subject, match an arm, and run its body.

use super::case_collect::{collect_case, CasePlan};
use super::foreach_run::run_body;
use super::line_edit::ReplInput;
use super::ReplIo;
use crate::case_block::{self, CaseHeader};
use crate::env::ShellEnvironment;
use crate::exec::CommandResult;
use crate::expand::expand_word_for_exec;

use std::io::{self, Write};

/// Collect arms and run the first matching body.
pub(super) fn run_case<I: ReplInput, O: Write, E: Write>(
    header: CaseHeader,
    io: &mut ReplIo<'_, I, O, E>,
    interactive: bool,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let Some(plan) = collect_case(
        io,
        interactive,
        &shell_env.history,
        &mut shell_env.key_bindings,
    )?
    else {
        return Ok(CommandResult::Status(1));
    };
    run_plan(header, plan, io, shell_env, last_status, argv)
}

fn run_plan<I: ReplInput, O: Write, E: Write>(
    header: CaseHeader,
    plan: CasePlan,
    io: &mut ReplIo<'_, I, O, E>,
    shell_env: &mut ShellEnvironment,
    last_status: u8,
    argv: &mut Vec<String>,
) -> io::Result<CommandResult> {
    let subject = match expand_word_for_exec(&header.subject, shell_env, last_status) {
        Ok(word) => word.into_string(),
        Err(err) => {
            writeln!(io.stderr, "{}", err.message())?;
            return Ok(CommandResult::Status(1));
        }
    };
    for arm in &plan.arms {
        if arm_matches(&subject, &arm.patterns, shell_env, last_status) {
            return run_body(&arm.body, io, shell_env, last_status, argv);
        }
    }
    Ok(CommandResult::Status(last_status))
}

fn arm_matches(
    subject: &str,
    patterns: &[String],
    shell_env: &ShellEnvironment,
    last_status: u8,
) -> bool {
    for raw in patterns {
        let pat = expand_word_for_exec(raw, shell_env, last_status)
            .map(|w| w.into_string())
            .unwrap_or_else(|_| raw.clone());
        if case_block::matches_any(subject, &pat) {
            return true;
        }
    }
    false
}
