//! Default emacs and vi key maps.

use super::{Action, Binding};

use std::collections::HashMap;

type Map = HashMap<Vec<u8>, Binding>;

fn put(map: &mut Map, keys: &[u8], action: Action) {
    map.insert(keys.to_vec(), Binding::Action(action));
}

pub(super) fn seed_emacs(primary: &mut Map, alternate: &mut Map) {
    seed_common(primary);
    seed_word_ops(primary);
    seed_line_motion(primary);
    put(primary, b"\x1b[D", Action::MoveLeft);
    put(primary, b"\x1b[C", Action::MoveRight);
    put(primary, b"\x1b[A", Action::HistoryUp);
    put(primary, b"\x1b[B", Action::HistoryDown);
    put(primary, b"\x1b[3~", Action::Delete);
    alternate.clear();
}

pub(super) fn seed_vi(primary: &mut Map, alternate: &mut Map) {
    seed_common(primary);
    seed_word_ops(primary);
    seed_line_motion(primary);
    put(primary, b"\x1b", Action::ViCmdMode);
    put(primary, b"\x1b[D", Action::MoveLeft);
    put(primary, b"\x1b[C", Action::MoveRight);
    put(primary, b"\x1b[A", Action::HistoryUp);
    put(primary, b"\x1b[B", Action::HistoryDown);
    put(primary, b"\x1b[3~", Action::Delete);

    alternate.clear();
    put(alternate, b"h", Action::MoveLeft);
    put(alternate, b"l", Action::MoveRight);
    put(alternate, b"0", Action::MoveHome);
    put(alternate, b"^", Action::MoveHome);
    put(alternate, b"$", Action::MoveEnd);
    put(alternate, b"b", Action::MoveWordLeft);
    put(alternate, b"w", Action::MoveWordRight);
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
    put(map, &[0x12], Action::HistoryISearch); // C-r
}

fn seed_line_motion(map: &mut Map) {
    put(map, &[0x01], Action::MoveHome); // C-a
    put(map, &[0x05], Action::MoveEnd); // C-e
    put(map, &[0x02], Action::MoveLeft); // C-b
    put(map, &[0x06], Action::MoveRight); // C-f
    put(map, b"\x1b[H", Action::MoveHome); // Home
    put(map, b"\x1b[F", Action::MoveEnd); // End
    put(map, b"\x1bOH", Action::MoveHome);
    put(map, b"\x1bOF", Action::MoveEnd);
}

fn seed_word_ops(map: &mut Map) {
    put(map, b"\x1bb", Action::MoveWordLeft); // M-b
    put(map, b"\x1bf", Action::MoveWordRight); // M-f
    put(map, b"\x1bd", Action::KillWordForward); // M-d
    put(map, b"\x1b\x7f", Action::KillWordBackward); // M-Backspace
    put(map, b"\x1b\x08", Action::KillWordBackward);
    put(map, &[0x17], Action::KillWordBackward); // C-w
    put(map, &[0x0b], Action::KillToEol); // C-k
    put(map, &[0x15], Action::KillLine); // C-u
    put(map, &[0x19], Action::Yank); // C-y
    put(map, b"\x1bt", Action::TransposeWords); // M-t
}
