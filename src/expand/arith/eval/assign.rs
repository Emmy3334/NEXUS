//! Assignment operators (`=` `+=` … `&=` `|=` `^=` `<<=` `>>=`).

use super::primary::{is_name_start, lookup_int, skip_ws, take_name};
use super::store::store;
use crate::env::ShellEnvironment;
use crate::lex::LexError;

use std::iter::Peekable;
use std::str::Chars;

enum Op {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Shl,
    Shr,
}

/// Right-associative assignment, else ternary.
pub(super) fn parse_assign(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<i64, LexError> {
    skip_ws(chars);
    let checkpoint = chars.clone();
    let Some(name) = take_lvalue_name(chars) else {
        *chars = checkpoint;
        return super::ternary::parse_ternary(chars, env, last_status);
    };
    skip_ws(chars);
    let Some(op) = take_assign_op(chars) else {
        *chars = checkpoint;
        return super::ternary::parse_ternary(chars, env, last_status);
    };
    let rhs = parse_assign(chars, env, last_status)?;
    let left = lookup_int(&name, env, last_status);
    let value = apply(op, left, rhs)?;
    store(env, &name, value);
    Ok(value)
}

fn take_lvalue_name(chars: &mut Peekable<Chars<'_>>) -> Option<String> {
    let first = chars.peek().copied()?;
    if !is_name_start(first) {
        return None;
    }
    Some(take_name(chars))
}

fn take_assign_op(chars: &mut Peekable<Chars<'_>>) -> Option<Op> {
    take_shift_assign(chars).or_else(|| take_two_char_assign(chars))
}

fn take_shift_assign(chars: &mut Peekable<Chars<'_>>) -> Option<Op> {
    let first = chars.peek().copied()?;
    let mut ahead = chars.clone();
    ahead.next();
    let second = ahead.peek().copied()?;
    ahead.next();
    let third = ahead.peek().copied()?;
    let op = match (first, second, third) {
        ('<', '<', '=') => Op::Shl,
        ('>', '>', '=') => Op::Shr,
        _ => return None,
    };
    chars.next();
    chars.next();
    chars.next();
    Some(op)
}

fn take_two_char_assign(chars: &mut Peekable<Chars<'_>>) -> Option<Op> {
    let first = chars.peek().copied()?;
    let mut ahead = chars.clone();
    ahead.next();
    let second = ahead.peek().copied();
    let op = match (first, second) {
        ('=', Some('=')) | ('+', Some('+')) | ('-', Some('-')) => return None,
        ('=', _) => Op::Set,
        ('+', Some('=')) => Op::Add,
        ('-', Some('=')) => Op::Sub,
        ('*', Some('=')) => Op::Mul,
        ('/', Some('=')) => Op::Div,
        ('%', Some('=')) => Op::Rem,
        ('&', Some('=')) => Op::And,
        ('|', Some('=')) => Op::Or,
        ('^', Some('=')) => Op::Xor,
        _ => return None,
    };
    chars.next();
    if !matches!(op, Op::Set) {
        chars.next();
    }
    Some(op)
}

fn apply(op: Op, left: i64, right: i64) -> Result<i64, LexError> {
    let shl = |n: i64| u32::try_from(n.clamp(0, 63)).unwrap_or(0);
    match op {
        Op::Set => Ok(right),
        Op::Add => Ok(left.wrapping_add(right)),
        Op::Sub => Ok(left.wrapping_sub(right)),
        Op::Mul => Ok(left.wrapping_mul(right)),
        Op::Div if right == 0 => Err(LexError::Arithmetic),
        Op::Div => Ok(left.wrapping_div(right)),
        Op::Rem if right == 0 => Err(LexError::Arithmetic),
        Op::Rem => Ok(left.wrapping_rem(right)),
        Op::And => Ok(left & right),
        Op::Or => Ok(left | right),
        Op::Xor => Ok(left ^ right),
        Op::Shl => Ok(left.wrapping_shl(shl(right))),
        Op::Shr => Ok(left.wrapping_shr(shl(right))),
    }
}
