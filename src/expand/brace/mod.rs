//! Bash/zsh-style brace expansion (`{a,b}`) before `$` / pathname glob.

mod find;
mod parse;

/// Expand unquoted `{a,b}` forms. No commas / invalid braces → one literal.
#[must_use]
pub(super) fn expand(raw: &str) -> Vec<String> {
    match find::first_brace(raw) {
        None => vec![raw.to_owned()],
        Some((prefix, alts, suffix)) => {
            let mut out = Vec::new();
            for alt in alts {
                let combined = format!("{prefix}{alt}{suffix}");
                out.extend(expand(&combined));
            }
            out
        }
    }
}
