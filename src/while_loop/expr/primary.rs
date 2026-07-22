//! Primary / brace / comparison tails of a while expression.

use super::brace;
use super::compare::{eval_cmp, truthy};
use super::eval::{self, Ctx};

use std::io::{BufRead, Write};

pub(super) fn parse_primary<I: BufRead, E: Write>(
    parts: &[String],
    i: &mut usize,
    ctx: &mut Ctx<'_, I, E>,
) -> Result<bool, String> {
    let Some(tok) = parts.get(*i) else {
        return Err("while: Expression Syntax.".into());
    };
    if tok == "(" {
        *i += 1;
        let value = eval::parse_or(parts, i, ctx)?;
        if parts.get(*i).map(String::as_str) != Some(")") {
            return Err("while: Expression Syntax.".into());
        }
        *i += 1;
        return Ok(value);
    }
    if tok == "{" {
        return parse_brace(parts, i, ctx);
    }
    parse_value_or_cmp(parts, i)
}

fn parse_brace<I: BufRead, E: Write>(
    parts: &[String],
    i: &mut usize,
    ctx: &mut Ctx<'_, I, E>,
) -> Result<bool, String> {
    *i += 1;
    let start = *i;
    while parts.get(*i).map(String::as_str) != Some("}") {
        if *i >= parts.len() {
            return Err("while: Expression Syntax.".into());
        }
        *i += 1;
    }
    let words = &parts[start..*i];
    *i += 1;
    brace::eval_brace(words, ctx)
}

fn parse_value_or_cmp(parts: &[String], i: &mut usize) -> Result<bool, String> {
    let left = parts
        .get(*i)
        .ok_or_else(|| "while: Expression Syntax.".to_owned())?;
    *i += 1;
    let Some(op) = parts.get(*i).map(String::as_str) else {
        return Ok(truthy(left));
    };
    if !matches!(op, "==" | "!=" | "<" | ">" | "<=" | ">=") {
        return Ok(truthy(left));
    }
    *i += 1;
    let right = parts
        .get(*i)
        .ok_or_else(|| "while: Expression Syntax.".to_owned())?;
    *i += 1;
    eval_cmp(left, op, right)
}
