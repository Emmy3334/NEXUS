//! Case-insensitive prefix and substring matching.

/// True when `candidate` matches `prefix` (exact, case-insensitive prefix, or substring).
#[must_use]
pub fn matches_prefix(candidate: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    if candidate.starts_with(prefix) {
        return true;
    }
    let c = candidate.to_ascii_lowercase();
    let p = prefix.to_ascii_lowercase();
    if c.starts_with(&p) {
        return true;
    }
    prefix.len() >= 2 && c.contains(&p)
}

/// Prefix-only match for filesystem names (zsh-style: no substring fuzz).
///
/// File/dir listings match by leading characters only so `cd ta` never offers
/// `StartupFiles/` or `Git_Mistakes.pdf`; case-insensitive to stay ergonomic.
#[must_use]
pub fn matches_file_prefix(candidate: &str, prefix: &str) -> bool {
    if prefix.is_empty() {
        return true;
    }
    candidate.starts_with(prefix)
        || candidate
            .to_ascii_lowercase()
            .starts_with(&prefix.to_ascii_lowercase())
}

/// Score how strongly `candidate` matches `prefix` (higher is better).
#[must_use]
pub fn prefix_score(candidate: &str, prefix: &str) -> i32 {
    if prefix.is_empty() {
        return 0;
    }
    if candidate.starts_with(prefix) {
        return 100;
    }
    let c = candidate.to_ascii_lowercase();
    let p = prefix.to_ascii_lowercase();
    if c.starts_with(&p) {
        return 90;
    }
    if prefix.len() >= 2 && c.contains(&p) {
        return 50;
    }
    0
}
