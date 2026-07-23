//! Beginning/end-of-line and char-motion default bindings.

use nexus::keybind::{action_name, parse_action, Action, Binding, KeyBindings};

#[test]
fn emacs_defaults_include_line_motion() {
    let bindings = KeyBindings::new();
    assert_eq!(
        bindings.lookup(&[0x01]),
        Some(&Binding::Action(Action::MoveHome))
    );
    assert_eq!(
        bindings.lookup(&[0x05]),
        Some(&Binding::Action(Action::MoveEnd))
    );
    assert_eq!(
        bindings.lookup(&[0x02]),
        Some(&Binding::Action(Action::MoveLeft))
    );
    assert_eq!(
        bindings.lookup(&[0x06]),
        Some(&Binding::Action(Action::MoveRight))
    );
    assert_eq!(
        bindings.lookup(b"\x1b[H"),
        Some(&Binding::Action(Action::MoveHome))
    );
    assert_eq!(
        bindings.lookup(b"\x1b[F"),
        Some(&Binding::Action(Action::MoveEnd))
    );
    assert_eq!(
        bindings.lookup(b"\x1bOH"),
        Some(&Binding::Action(Action::MoveHome))
    );
    assert_eq!(
        bindings.lookup(b"\x1bOF"),
        Some(&Binding::Action(Action::MoveEnd))
    );
    assert_eq!(
        bindings.lookup(&[0x0c]),
        Some(&Binding::Action(Action::ClearScreen))
    );
}

#[test]
fn vi_command_map_has_bol_eol() {
    let mut bindings = KeyBindings::new();
    bindings.reset_vi();
    bindings.enter_command_map();
    assert_eq!(
        bindings.lookup(b"0"),
        Some(&Binding::Action(Action::MoveHome))
    );
    assert_eq!(
        bindings.lookup(b"^"),
        Some(&Binding::Action(Action::MoveHome))
    );
    assert_eq!(
        bindings.lookup(b"$"),
        Some(&Binding::Action(Action::MoveEnd))
    );
}

#[test]
fn line_motion_action_names_round_trip() {
    for (name, action) in [
        ("beginning-of-line", Action::MoveHome),
        ("end-of-line", Action::MoveEnd),
        ("backward-char", Action::MoveLeft),
        ("forward-char", Action::MoveRight),
        ("clear-screen", Action::ClearScreen),
    ] {
        assert_eq!(parse_action(name), Some(action), "parse {name}");
        assert_eq!(action_name(action), name, "name {action:?}");
    }
}
