//! Prefix and approximate matchers for completion.

mod approx;
mod prefix;

pub use approx::approx_matches;
pub use prefix::{matches_prefix, prefix_score};
