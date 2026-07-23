//! Dispatch parsed `bindkey` invocations.

use super::args::Invocation;
use super::{set, usage};
use crate::env::ShellEnvironment;
use crate::keybind::{self, Action};

use std::io::{self, Write};

pub(super) fn run(
    inv: Invocation,
    shell_env: &mut ShellEnvironment,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match inv {
        Invocation::Usage => usage::print(stdout),
        Invocation::ListCommands => list_commands(stdout),
        Invocation::ResetDefault => {
            shell_env.key_bindings.reset_default();
            Ok(0)
        }
        Invocation::ResetEmacs => {
            shell_env.key_bindings.reset_emacs();
            Ok(0)
        }
        Invocation::ResetVi => {
            shell_env.key_bindings.reset_vi();
            Ok(0)
        }
        Invocation::List { alternate } => list_all(shell_env, alternate, stdout),
        Invocation::Show { keys, alternate } => {
            show_one(shell_env, &keys, alternate, stdout, stderr)
        }
        Invocation::Remove { keys, alternate } => remove_one(shell_env, &keys, alternate, stderr),
        Invocation::Bind {
            keys,
            value,
            alternate,
            kind,
        } => set::apply(shell_env, keys, value, alternate, kind, stderr),
    }
}

fn list_all(
    shell_env: &ShellEnvironment,
    alternate: bool,
    stdout: &mut impl Write,
) -> io::Result<u8> {
    for (keys, binding) in shell_env.key_bindings.sorted_entries(alternate) {
        writeln!(
            stdout,
            "\"{}\"\t{}",
            keybind::format_key(&keys),
            keybind::binding_label(&binding)
        )?;
    }
    Ok(0)
}

fn list_commands(stdout: &mut impl Write) -> io::Result<u8> {
    for action in ALL_ACTIONS {
        writeln!(stdout, "{}", keybind::action_name(action))?;
    }
    Ok(0)
}

fn show_one(
    shell_env: &ShellEnvironment,
    keys: &[u8],
    alternate: bool,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    match shell_env.key_bindings.lookup_in(keys, alternate) {
        Some(binding) => {
            writeln!(stdout, "{}", keybind::binding_label(binding))?;
            Ok(0)
        }
        None => {
            writeln!(stderr, "bindkey: Unbound key.")?;
            Ok(1)
        }
    }
}

fn remove_one(
    shell_env: &mut ShellEnvironment,
    keys: &[u8],
    alternate: bool,
    stderr: &mut impl Write,
) -> io::Result<u8> {
    if shell_env.key_bindings.unbind(keys, alternate) {
        Ok(0)
    } else {
        writeln!(stderr, "bindkey: Unbound key.")?;
        Ok(1)
    }
}

const ALL_ACTIONS: [Action; 20] = [
    Action::Accept,
    Action::Backspace,
    Action::Delete,
    Action::MoveLeft,
    Action::MoveRight,
    Action::MoveWordLeft,
    Action::MoveWordRight,
    Action::KillWordForward,
    Action::KillWordBackward,
    Action::KillToEol,
    Action::KillLine,
    Action::Yank,
    Action::TransposeWords,
    Action::HistoryUp,
    Action::HistoryDown,
    Action::Complete,
    Action::Interrupt,
    Action::Eof,
    Action::ViCmdMode,
    Action::ViInsertMode,
];
