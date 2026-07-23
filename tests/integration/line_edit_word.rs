//! Word-motion / kill-ring default bindings and action names.

use nexus::keybind::{action_name, parse_action, Action, Binding, KeyBindings};

#[test]
fn emacs_defaults_include_word_ops() {
    let bindings = KeyBindings::new();
    assert_eq!(
        bindings.lookup(b"\x1bb"),
        Some(&Binding::Action(Action::MoveWordLeft))
    );
    assert_eq!(
        bindings.lookup(b"\x1bf"),
        Some(&Binding::Action(Action::MoveWordRight))
    );
    assert_eq!(
        bindings.lookup(&[0x17]),
        Some(&Binding::Action(Action::KillWordBackward))
    );
    assert_eq!(
        bindings.lookup(b"\x1bd"),
        Some(&Binding::Action(Action::KillWordForward))
    );
    assert_eq!(
        bindings.lookup(&[0x0b]),
        Some(&Binding::Action(Action::KillToEol))
    );
    assert_eq!(
        bindings.lookup(&[0x15]),
        Some(&Binding::Action(Action::KillLine))
    );
    assert_eq!(
        bindings.lookup(&[0x19]),
        Some(&Binding::Action(Action::Yank))
    );
    assert_eq!(
        bindings.lookup(b"\x1bt"),
        Some(&Binding::Action(Action::TransposeWords))
    );
}

#[test]
fn vi_command_map_has_word_motion() {
    let mut bindings = KeyBindings::new();
    bindings.reset_vi();
    bindings.enter_command_map();
    assert_eq!(
        bindings.lookup(b"b"),
        Some(&Binding::Action(Action::MoveWordLeft))
    );
    assert_eq!(
        bindings.lookup(b"w"),
        Some(&Binding::Action(Action::MoveWordRight))
    );
}

#[test]
fn word_action_names_round_trip() {
    for (name, action) in [
        ("backward-word", Action::MoveWordLeft),
        ("forward-word", Action::MoveWordRight),
        ("kill-word", Action::KillWordForward),
        ("backward-kill-word", Action::KillWordBackward),
        ("kill-line", Action::KillToEol),
        ("kill-whole-line", Action::KillLine),
        ("yank", Action::Yank),
        ("transpose-words", Action::TransposeWords),
    ] {
        assert_eq!(parse_action(name), Some(action), "parse {name}");
        assert_eq!(action_name(action), name, "name {action:?}");
    }
    assert_eq!(
        parse_action("unix-word-rubout"),
        Some(Action::KillWordBackward)
    );
}
