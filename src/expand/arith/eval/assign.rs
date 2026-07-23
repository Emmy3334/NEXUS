//! Assignment operators at the top of `$((…))` (`=` `+=` `-=` `*=` `/=` `%=`).

use super::primary::{is_name_start, lookup_int, skip_ws, take_name};
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
    let first = chars.peek().copied()?;
    let mut ahead = chars.clone();
    ahead.next();
    let second = ahead.peek().copied();
    match (first, second) {
        ('=', Some('=')) | ('+', Some('+')) | ('-', Some('-')) => None,
        ('=', _) => {
            chars.next();
            Some(Op::Set)
        }
        ('+', Some('=')) => {
            chars.next();
            chars.next();
            Some(Op::Add)
        }
        ('-', Some('=')) => {
            chars.next();
            chars.next();
            Some(Op::Sub)
        }
        ('*', Some('=')) => {
            chars.next();
            chars.next();
            Some(Op::Mul)
        }
        ('/', Some('=')) => {
            chars.next();
            chars.next();
            Some(Op::Div)
        }
        ('%', Some('=')) => {
            chars.next();
            chars.next();
            Some(Op::Rem)
        }
        _ => None,
    }
}

fn apply(op: Op, left: i64, right: i64) -> Result<i64, LexError> {
    match op {
        Op::Set => Ok(right),
        Op::Add => Ok(left.wrapping_add(right)),
        Op::Sub => Ok(left.wrapping_sub(right)),
        Op::Mul => Ok(left.wrapping_mul(right)),
        Op::Div if right == 0 => Err(LexError::Arithmetic),
        Op::Div => Ok(left.wrapping_div(right)),
        Op::Rem if right == 0 => Err(LexError::Arithmetic),
        Op::Rem => Ok(left.wrapping_rem(right)),
    }
}

fn store(env: &mut ShellEnvironment, name: &str, value: i64) {
    let text = value.to_string();
    if env.get_local(name).is_some() {
        env.set_local(name, text);
    } else if env.contains(name) {
        env.set(name, text);
    } else {
        env.set_local(name, text);
    }
}
