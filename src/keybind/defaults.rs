//! Default emacs and vi key maps.

use super::{Action, Binding};

use std::collections::HashMap;

type Map = HashMap<Vec<u8>, Binding>;

fn put(map: &mut Map, keys: &[u8], action: Action) {
    map.insert(keys.to_vec(), Binding::Action(action));
}

pub(super) fn seed_emacs(primary: &mut Map, alternate: &mut Map) {
    seed_common(primary);
    put(primary, b"\x1b[D", Action::MoveLeft);
    put(primary, b"\x1b[C", Action::MoveRight);
    put(primary, b"\x1b[A", Action::HistoryUp);
    put(primary, b"\x1b[B", Action::HistoryDown);
    put(primary, b"\x1b[3~", Action::Delete);
    alternate.clear();
}

pub(super) fn seed_vi(primary: &mut Map, alternate: &mut Map) {
    seed_common(primary);
    put(primary, b"\x1b", Action::ViCmdMode);
    put(primary, b"\x1b[D", Action::MoveLeft);
    put(primary, b"\x1b[C", Action::MoveRight);
    put(primary, b"\x1b[A", Action::HistoryUp);
    put(primary, b"\x1b[B", Action::HistoryDown);
    put(primary, b"\x1b[3~", Action::Delete);

    alternate.clear();
    put(alternate, b"h", Action::MoveLeft);
    put(alternate, b"l", Action::MoveRight);
    put(alternate, b"k", Action::HistoryUp);
    put(alternate, b"j", Action::HistoryDown);
    put(alternate, b"i", Action::ViInsertMode);
    put(alternate, b"a", Action::ViInsertMode);
    put(alternate, b"\r", Action::Accept);
    put(alternate, b"\n", Action::Accept);
    put(alternate, b"\x03", Action::Interrupt);
}

fn seed_common(map: &mut Map) {
    put(map, b"\r", Action::Accept);
    put(map, b"\n", Action::Accept);
    put(map, &[0x7f], Action::Backspace);
    put(map, &[0x08], Action::Backspace);
    put(map, b"\t", Action::Complete);
    put(map, &[0x03], Action::Interrupt);
    put(map, &[0x04], Action::Eof);
}
