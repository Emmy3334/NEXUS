//! Prefix and approximate matchers for completion.

mod approx;
mod prefix;

pub use approx::approx_matches;
pub use prefix::{matches_file_prefix, matches_prefix, prefix_score};
