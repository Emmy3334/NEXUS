//! Prefix and postfix `++` / `--` on bare names in `$((…))`.

use super::super::store::store;
use super::{is_name_start, lookup_int, skip_ws, take_name};
use crate::env::ShellEnvironment;

use std::iter::Peekable;
use std::str::Chars;

/// `++name` / `--name` → new value after writeback.
pub(super) fn try_prefix(
    chars: &mut Peekable<Chars<'_>>,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Option<i64> {
    skip_ws(chars);
    let delta = take_inc(chars)?;
    skip_ws(chars);
    let c = chars.peek().copied()?;
    if !is_name_start(c) {
        return None;
    }
    let name = take_name(chars);
    let (_, new) = bump(&name, delta, env, last_status);
    Some(new)
}

/// After a bare name: `name++` / `name--` → old value after writeback.
pub(super) fn try_postfix(
    chars: &mut Peekable<Chars<'_>>,
    name: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Option<i64> {
    skip_ws(chars);
    let delta = take_inc(chars)?;
    let (old, _) = bump(name, delta, env, last_status);
    Some(old)
}

fn take_inc(chars: &mut Peekable<Chars<'_>>) -> Option<i64> {
    let first = chars.peek().copied()?;
    let mut ahead = chars.clone();
    ahead.next();
    let second = ahead.peek().copied()?;
    match (first, second) {
        ('+', '+') => {
            chars.next();
            chars.next();
            Some(1)
        }
        ('-', '-') => {
            chars.next();
            chars.next();
            Some(-1)
        }
        _ => None,
    }
}

fn bump(name: &str, delta: i64, env: &mut ShellEnvironment, last_status: u8) -> (i64, i64) {
    let old = lookup_int(name, env, last_status);
    let new = old.wrapping_add(delta);
    store(env, name, new);
    (old, new)
}
