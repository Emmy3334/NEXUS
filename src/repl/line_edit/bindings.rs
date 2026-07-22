//! Key-binding table (dynamic rebinding entry point).

use std::collections::HashMap;

/// Action performed when a key sequence is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Accept,
    Backspace,
    Delete,
    MoveLeft,
    MoveRight,
    HistoryUp,
    HistoryDown,
    Complete,
    Interrupt,
    Eof,
}

/// Mutable map from key bytes to editor actions.
#[derive(Debug, Clone)]
pub struct KeyBindings {
    map: HashMap<Vec<u8>, Action>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBindings {
    #[must_use]
    pub fn new() -> Self {
        let mut map = HashMap::new();
        seed_defaults(&mut map);
        Self { map }
    }

    /// Rebind `keys` to `action` (dynamic rebinding).
    pub fn bind(&mut self, keys: impl Into<Vec<u8>>, action: Action) {
        self.map.insert(keys.into(), action);
    }

    #[must_use]
    pub fn lookup(&self, keys: &[u8]) -> Option<Action> {
        self.map.get(keys).copied()
    }
}

fn seed_defaults(map: &mut HashMap<Vec<u8>, Action>) {
    map.insert(vec![b'\r'], Action::Accept);
    map.insert(vec![b'\n'], Action::Accept);
    map.insert(vec![0x7f], Action::Backspace);
    map.insert(vec![0x08], Action::Backspace);
    map.insert(vec![b'\t'], Action::Complete);
    map.insert(vec![0x03], Action::Interrupt);
    map.insert(vec![0x04], Action::Eof);
    map.insert(b"\x1b[D".to_vec(), Action::MoveLeft);
    map.insert(b"\x1b[C".to_vec(), Action::MoveRight);
    map.insert(b"\x1b[A".to_vec(), Action::HistoryUp);
    map.insert(b"\x1b[B".to_vec(), Action::HistoryDown);
    map.insert(b"\x1b[3~".to_vec(), Action::Delete);
}
