//! Host spawn hardening: PATH jail, child rlimit, and trusted-bin allowlist.

mod config;
pub(crate) mod rlimit;
mod trusted;

pub use config::{effective_path, path_jail_enabled, rlimit_enabled};
pub use trusted::{allowlist as trusted_bin_allowlist, check as check_trusted};
