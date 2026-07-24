//! Unique prefix among directory siblings.

use std::fs;
use std::path::Path;

#[must_use]
pub fn unique_prefix(name: &str, parent: Option<&Path>) -> String {
    let Some(dir) = parent else {
        return name.chars().take(1).collect();
    };
    let siblings = read_names(dir);
    for n in 1..=name.chars().count() {
        let prefix: String = name.chars().take(n).collect();
        let clash = siblings
            .iter()
            .any(|s| s.as_str() != name && s.starts_with(&prefix));
        if !clash {
            return prefix;
        }
    }
    name.to_owned()
}

fn read_names(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .collect()
}

pub fn enforce_max(parts: &mut [String], last: usize, max_len: usize) {
    while parts.join("/").chars().count() > max_len {
        let Some(i) = (0..last).find(|&i| parts[i].chars().count() > 1) else {
            break;
        };
        parts[i] = parts[i].chars().take(1).collect();
    }
}
