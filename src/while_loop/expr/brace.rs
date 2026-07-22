//! `{ command… }` status → 1/0 in while expressions.

use super::eval::Ctx;
use crate::exec::{self, CommandResult};
use crate::lex;
use crate::parse;

use std::io::{BufRead, Write};

pub(super) fn eval_brace<I: BufRead, E: Write>(
    words: &[String],
    ctx: &mut Ctx<'_, I, E>,
) -> Result<bool, String> {
    if words.is_empty() {
        return Err("while: Expression Syntax.".into());
    }
    let line = words.join(" ");
    let mut tokens = Vec::new();
    lex::tokenize_into(&line, &mut tokens).map_err(|err| err.message().to_owned())?;
    let list = match parse::parse_line(&line, &tokens) {
        Ok(Some(list)) => list,
        Ok(None) => return Ok(true),
        Err(err) => return Err(err.message().to_owned()),
    };
    let mut argv = Vec::new();
    let mut sink = Vec::new();
    let result = exec::execute_list(
        &list,
        &mut argv,
        ctx.env,
        ctx.last_status,
        Vec::new(),
        ctx.stdin,
        &mut sink,
        ctx.stderr,
    )
    .map_err(|err| err.to_string())?;
    Ok(matches!(result, CommandResult::Status(0)))
}
