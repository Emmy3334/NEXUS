//! Quote-aware word expansion: escapes, `$`, and `` ` `` (glob flags recorded).
//!
//! Pathname expansion lives in [`crate::glob`].
//! Command substitution capture is injected by callers (avoids expand↔exec cycle).

mod backtick;
mod braced;
mod decode;
mod dollar;
mod fields;
mod name;
mod push;
mod word;

pub use decode::{expand_word_fields_into, expand_word_for_exec, expand_word_for_exec_into};
pub use word::ExpandedWord;
