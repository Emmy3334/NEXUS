//! Boolean expression parser for `[[ … ]]`.

use super::{binary, unary};

pub(super) fn eval(tokens: &[String]) -> Result<bool, &'static str> {
    or_expr(tokens)
}

fn or_expr(tokens: &[String]) -> Result<bool, &'static str> {
    let parts = split(tokens, "||");
    let mut result = false;
    for part in parts {
        result = result || and_expr(part)?;
    }
    Ok(result)
}

fn and_expr(tokens: &[String]) -> Result<bool, &'static str> {
    let parts = split(tokens, "&&");
    let mut result = true;
    for part in parts {
        result = result && primary(part)?;
    }
    Ok(result)
}

fn primary(tokens: &[String]) -> Result<bool, &'static str> {
    match tokens {
        [] => Ok(false),
        [op, arg] if op.starts_with('-') && op.len() == 2 => unary::eval(op, arg),
        [left, op, right] => binary::eval(left, op, right),
        [one] => Ok(!one.is_empty()),
        _ => Err("parse error"),
    }
}

fn split<'a>(tokens: &'a [String], separator: &str) -> Vec<&'a [String]> {
    let mut parts = Vec::new();
    let mut start = 0;
    for (i, token) in tokens.iter().enumerate() {
        if token == separator {
            parts.push(&tokens[start..i]);
            start = i + 1;
        }
    }
    parts.push(&tokens[start..]);
    parts
}
