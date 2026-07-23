//! Shared helpers for exec integration tests.

use nexus::env::ShellEnvironment;
use nexus::parse::CommandList;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Serialize tests that mutate process-global `current_dir`.
pub static CWD_LOCK: Mutex<()> = Mutex::new(());

pub fn test_env() -> ShellEnvironment {
    let path = std::env::var("PATH").unwrap_or_default();
    let mut map = BTreeMap::new();
    map.insert("PATH".into(), path);
    ShellEnvironment::from_map(map)
}

pub fn parse_list(source: &str) -> CommandList<'_> {
    let mut tokens = Vec::new();
    nexus::lex::tokenize_into(source, &mut tokens).expect("lex ok");
    nexus::parse::parse_line(source, &tokens)
        .expect("parse ok")
        .expect("non-empty list")
}
