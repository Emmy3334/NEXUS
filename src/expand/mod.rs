//! Quote-aware word expansion: escapes and `$` parameters (glob flags recorded).
//!
//! Pathname expansion lives in [`crate::glob`].

mod decode;
mod dollar;
mod name;
mod push;
mod word;

pub use decode::{expand_word_for_exec, expand_word_for_exec_into};
pub use word::ExpandedWord;
