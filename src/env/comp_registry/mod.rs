//! User-defined completion words (`compdef` / `compinit` / `compdump`).

mod audit;
mod dump;
mod parse;

pub use dump::{boot_load, dump_path_is_safe, load_dump, resolve_dump_path, save_dump};

use std::collections::BTreeMap;

/// On-disk dump format version.
pub const DUMP_VERSION: u32 = 1;

/// Static first-verb words registered via `compdef`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompRegistry {
    pub(crate) words: BTreeMap<String, Vec<String>>,
}

impl CompRegistry {
    /// Whether `cmd` has registered completion words.
    #[must_use]
    pub fn has_cmd(&self, cmd: &str) -> bool {
        self.words.contains_key(cmd)
    }

    /// Replace the word list for `cmd`.
    pub fn register(&mut self, cmd: &str, words: Vec<String>) {
        self.words.insert(cmd.to_owned(), words);
    }

    /// Remove all registered words for `cmd`.
    pub fn delete(&mut self, cmd: &str) -> bool {
        self.words.remove(cmd).is_some()
    }

    /// Collect registered words for `cmd` matching `prefix` (case-insensitive).
    pub fn collect(&self, cmd: &str, prefix: &str, out: &mut Vec<String>) {
        let Some(words) = self.words.get(cmd) else {
            return;
        };
        for word in words {
            if prefix_matches(word, prefix) {
                out.push(word.clone());
            }
        }
    }
}

fn prefix_matches(candidate: &str, prefix: &str) -> bool {
    candidate
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}
