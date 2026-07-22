//! Finish parsing positional key / command args.

use super::{BindKind, Flags, Invocation};
use crate::keybind::{parse_key, parse_key_b};

pub(in crate::builtins::bindkey) fn finish(
    flags: Flags,
    rest: &[String],
) -> Result<Invocation, String> {
    if rest.is_empty() {
        if flags.remove || flags.arrow || flags.kind != BindKind::Editor {
            return Err("Missing key.".into());
        }
        return Ok(Invocation::List {
            alternate: flags.alternate,
        });
    }
    let keys = resolve_keys(&flags, &rest[0])?;
    match rest.len() {
        1 if flags.remove => Ok(Invocation::Remove {
            keys,
            alternate: flags.alternate,
        }),
        1 => Ok(Invocation::Show {
            keys,
            alternate: flags.alternate,
        }),
        2 if flags.remove => Err("Too many arguments.".into()),
        2 => Ok(Invocation::Bind {
            keys,
            value: rest[1].clone(),
            alternate: flags.alternate,
            kind: flags.kind,
        }),
        _ => Err("Too many arguments.".into()),
    }
}

fn resolve_keys(flags: &Flags, spec: &str) -> Result<Vec<u8>, String> {
    if flags.arrow {
        return arrow_keys(spec).ok_or_else(|| "Bad arrow key name.".into());
    }
    let parsed = if flags.bind_b {
        parse_key_b(spec)
    } else {
        parse_key(spec)
    };
    parsed.ok_or_else(|| "Bad key specification.".into())
}

fn arrow_keys(name: &str) -> Option<Vec<u8>> {
    Some(
        match name {
            "left" => b"\x1b[D",
            "right" => b"\x1b[C",
            "up" => b"\x1b[A",
            "down" => b"\x1b[B",
            _ => return None,
        }
        .to_vec(),
    )
}
