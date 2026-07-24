//! Pathname expansion (`*`, `?`, `[…]`, `**`) after quote / `$` expansion.
//!
//! Only characters flagged active in [`ExpandedWord`] participate as metas.
//! No match → the literal word is kept (bash / tcsh-`nonomatch` style).

mod bracket;
mod component;
mod components;
mod globstar;
mod match_name;
mod qualifier;
mod walk;

pub use walk::{expand_globs, expand_globs_one, AmbiguousGlob};
