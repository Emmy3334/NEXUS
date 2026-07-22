//! Apply a `bindkey` bind operation.

use super::args::BindKind;
use crate::env::ShellEnvironment;
use crate::keybind::{self, Binding};

use std::io::{self, Write};

pub(super) fn apply(
    shell_env: &mut ShellEnvironment,
    keys: Vec<u8>,
    value: String,
    alternate: bool,
    kind: BindKind,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    let binding = match kind {
        BindKind::Editor => match keybind::parse_action(&value) {
            Some(action) => Binding::Action(action),
            None => {
                writeln!(stderr, "bindkey: Unknown editor command.")?;
                return Ok(1);
            }
        },
        BindKind::Command => Binding::Command(value),
        BindKind::Literal => Binding::Literal(value),
    };
    shell_env.key_bindings.bind(keys, binding, alternate);
    Ok(0)
}
