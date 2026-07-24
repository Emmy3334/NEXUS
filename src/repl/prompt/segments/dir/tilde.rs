//! Apply `~` when the absolute path is under `$home`.

use super::shorten::normal_names;
use std::path::Path;

#[must_use]
pub fn with_tilde(parts: &[String], home: &str, abs: &Path) -> String {
    if home.is_empty() {
        return format_abs(parts, abs);
    }
    let home_path = Path::new(home);
    if abs == home_path {
        return "~".into();
    }
    if let Ok(rel) = abs.strip_prefix(home_path) {
        let n = normal_names(rel).len();
        let start = parts.len().saturating_sub(n);
        return format!("~/{}", parts[start..].join("/"));
    }
    format_abs(parts, abs)
}

fn format_abs(parts: &[String], abs: &Path) -> String {
    if abs.is_absolute() {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::with_tilde;
    use std::path::Path;

    #[test]
    fn tilde_for_home_subdir() {
        let parts = vec!["Users".into(), "x".into(), "Doc".into()];
        let abs = Path::new("/Users/x/Doc");
        assert_eq!(with_tilde(&parts, "/Users/x", abs), "~/Doc");
    }
}
