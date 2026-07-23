//! Quote-aware word expansion: escapes, `$`, `` ` ``, and `$(…)` (glob flags recorded).
//!
//! Pathname expansion lives in [`crate::glob`].
//! Command substitution capture is injected by callers (avoids expand↔exec cycle).
//! Brace `{a,b}` runs in [`brace`] before these entry points.

mod arith;
mod backtick;
mod brace;
mod braced;
mod cmd_subst;
mod decode;
mod dollar;
mod fields;
mod name;
mod push;
mod subst_out;
mod word;

use crate::env::ShellEnvironment;
use crate::lex::LexError;

pub use word::ExpandedWord;

/// Expand a raw word into one field (no command-substitution capture).
pub fn expand_word_for_exec(
    raw: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
) -> Result<ExpandedWord, LexError> {
    let mut fields = Vec::new();
    let mut deny = |_: &str| Err(LexError::CommandSubstitution);
    expand_word_fields_into(raw, env, last_status, &mut fields, &mut deny)?;
    Ok(fields.into_iter().next().unwrap_or_default())
}

/// Expand `raw` into one or more fields (brace, then `$` / backticks / `$(…)`).
pub fn expand_word_fields_into(
    raw: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    fields_out: &mut Vec<ExpandedWord>,
    capture: &mut dyn FnMut(&str) -> Result<String, LexError>,
) -> Result<(), LexError> {
    fields_out.clear();
    for piece in brace::expand(raw) {
        let mut part = Vec::new();
        decode::expand_word_fields_into(&piece, env, last_status, &mut part, capture)?;
        fields_out.extend(part);
    }
    Ok(())
}

/// Like [`expand_word_for_exec`], writing into `out` (cleared first).
pub fn expand_word_for_exec_into(
    raw: &str,
    env: &mut ShellEnvironment,
    last_status: u8,
    out: &mut ExpandedWord,
) -> Result<(), LexError> {
    *out = expand_word_for_exec(raw, env, last_status)?;
    Ok(())
}
