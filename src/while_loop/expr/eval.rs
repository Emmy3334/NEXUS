//! Recursive-descent evaluation of normalized while expression tokens.

use super::primary;
use crate::env::ShellEnvironment;

use std::io::{BufRead, Write};

pub(super) struct Ctx<'a, I, E> {
    pub(super) env: &'a mut ShellEnvironment,
    pub(super) last_status: u8,
    pub(super) stdin: &'a mut I,
    pub(super) stderr: &'a mut E,
}

pub(super) fn eval_expr<I: BufRead, E: Write>(
    parts: &[String],
    ctx: &mut Ctx<'_, I, E>,
) -> Result<bool, String> {
    let mut i = 0;
    let value = parse_or(parts, &mut i, ctx)?;
    if i != parts.len() {
        return Err("while: Expression Syntax.".into());
    }
    Ok(value)
}

pub(super) fn parse_or<I: BufRead, E: Write>(
    parts: &[String],
    i: &mut usize,
    ctx: &mut Ctx<'_, I, E>,
) -> Result<bool, String> {
    let mut value = parse_and(parts, i, ctx)?;
    while parts.get(*i).map(String::as_str) == Some("||") {
        *i += 1;
        let rhs = parse_and(parts, i, ctx)?;
        value = value || rhs;
    }
    Ok(value)
}

fn parse_and<I: BufRead, E: Write>(
    parts: &[String],
    i: &mut usize,
    ctx: &mut Ctx<'_, I, E>,
) -> Result<bool, String> {
    let mut value = parse_unary(parts, i, ctx)?;
    while parts.get(*i).map(String::as_str) == Some("&&") {
        *i += 1;
        let rhs = parse_unary(parts, i, ctx)?;
        value = value && rhs;
    }
    Ok(value)
}

fn parse_unary<I: BufRead, E: Write>(
    parts: &[String],
    i: &mut usize,
    ctx: &mut Ctx<'_, I, E>,
) -> Result<bool, String> {
    if parts.get(*i).map(String::as_str) == Some("!") {
        *i += 1;
        return Ok(!parse_unary(parts, i, ctx)?);
    }
    primary::parse_primary(parts, i, ctx)
}
