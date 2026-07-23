//! Evaluate a tcsh-style while condition after `$` expansion.

mod brace;
mod compare;
mod eval;
mod normalize;
mod primary;

use crate::env::ShellEnvironment;
use crate::expand;

use std::io::{BufRead, Write};

/// Expand `expr` pieces and return whether the condition is true.
///
/// Writes a diagnostic to `stderr` and returns `None` on syntax errors.
pub fn eval_condition(
    expr: &[String],
    env: &mut ShellEnvironment,
    last_status: u8,
    stdin: &mut impl BufRead,
    stderr: &mut impl Write,
) -> Option<bool> {
    let expanded = match expand_pieces(expr, env, last_status) {
        Ok(parts) => normalize::merge_ops(parts),
        Err(msg) => {
            let _ = writeln!(stderr, "{msg}");
            return None;
        }
    };
    let mut ctx = eval::Ctx {
        env,
        last_status,
        stdin,
        stderr,
    };
    match eval::eval_expr(&expanded, &mut ctx) {
        Ok(value) => Some(value),
        Err(msg) => {
            let _ = writeln!(ctx.stderr, "{msg}");
            None
        }
    }
}

fn expand_pieces(
    expr: &[String],
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<Vec<String>, String> {
    let mut out = Vec::with_capacity(expr.len());
    for raw in expr {
        let word = expand::expand_word_for_exec(raw, env, last_status)
            .map_err(|e| e.message().to_owned())?;
        out.push(word.into_string());
    }
    Ok(out)
}
