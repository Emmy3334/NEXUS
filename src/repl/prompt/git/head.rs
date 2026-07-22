//! Parse `.git/HEAD` into a branch name or short detached SHA.

use std::fs;
use std::path::Path;

pub(super) fn branch_name(git_dir: &Path) -> Option<String> {
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref: ") {
        return Some(short_ref(reference.trim()));
    }
    if head.is_empty() {
        return None;
    }
    Some(head.chars().take(7).collect())
}

fn short_ref(reference: &str) -> String {
    reference
        .strip_prefix("refs/heads/")
        .or_else(|| reference.strip_prefix("refs/"))
        .unwrap_or(reference)
        .to_owned()
}
