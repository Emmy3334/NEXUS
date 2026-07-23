//! Host spawn hardening: PATH jail knobs and child rlimit config.

mod config;
pub(crate) mod rlimit;

pub use config::{effective_path, path_jail_enabled, rlimit_enabled};
