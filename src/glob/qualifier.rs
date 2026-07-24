//! Strip zsh-style trailing glob qualifiers and filter matches.

use std::path::Path;

/// Trailing qualifier on an active glob pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qualifier {
    RegularFile,
    Directory,
    Executable,
}

/// If `pattern` ends with `(.)`, `(/)`, or `(*)`, strip it and return the qualifier.
pub fn strip_qualifier(pattern: &[(char, bool)]) -> (Vec<(char, bool)>, Option<Qualifier>) {
    let Some(qual) = parse_suffix(pattern) else {
        return (pattern.to_vec(), None);
    };
    let trimmed = pattern[..pattern.len() - qual.suffix_len].to_vec();
    (trimmed, Some(qual.kind))
}

struct ParsedQual {
    kind: Qualifier,
    suffix_len: usize,
}

fn parse_suffix(pattern: &[(char, bool)]) -> Option<ParsedQual> {
    let tail: String = pattern.iter().map(|(c, _)| *c).collect();
    if tail.ends_with("(.)") {
        return Some(ParsedQual {
            kind: Qualifier::RegularFile,
            suffix_len: "(.)".chars().count(),
        });
    }
    if tail.ends_with("(/)") {
        return Some(ParsedQual {
            kind: Qualifier::Directory,
            suffix_len: "(/)".chars().count(),
        });
    }
    if tail.ends_with("(*)") {
        return Some(ParsedQual {
            kind: Qualifier::Executable,
            suffix_len: "(*)".chars().count(),
        });
    }
    None
}

/// Keep only paths that satisfy `qual` (relative to process cwd).
pub fn filter_matches(paths: Vec<String>, qual: Qualifier) -> Vec<String> {
    paths
        .into_iter()
        .filter(|p| path_matches(p, qual))
        .collect()
}

fn path_matches(path: &str, qual: Qualifier) -> bool {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    match qual {
        Qualifier::RegularFile => meta.is_file(),
        Qualifier::Directory => meta.is_dir(),
        Qualifier::Executable => is_executable(&meta, path),
    }
}

#[cfg(unix)]
fn is_executable(meta: &std::fs::Metadata, path: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if meta.is_dir() {
        return false;
    }
    if meta.permissions().mode() & 0o111 != 0 {
        return true;
    }
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext == "exe" || ext == "EXE")
}

#[cfg(not(unix))]
fn is_executable(_meta: &std::fs::Metadata, path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext == "exe" || ext == "EXE")
}
