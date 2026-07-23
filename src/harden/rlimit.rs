//! Opt-in child `setrlimit` (CPU / NOFILE; AS + NPROC on Linux).

use crate::env::ShellEnvironment;
use crate::harden::config::rlimit_enabled;

use nix::sys::resource::{setrlimit, Resource};
use std::io;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChildLimits {
    pub cpu_secs: u64,
    pub nofile: u64,
    /// Address space — Linux only (macOS rejects finite `RLIMIT_AS` with EINVAL).
    #[cfg(target_os = "linux")]
    pub as_bytes: u64,
    #[cfg(target_os = "linux")]
    pub nproc: u64,
}

const DEFAULT_CPU: u64 = 30;
const DEFAULT_NOFILE: u64 = 256;
#[cfg(target_os = "linux")]
const DEFAULT_AS: u64 = 512 * 1024 * 1024;
#[cfg(target_os = "linux")]
const DEFAULT_NPROC: u64 = 64;

/// Resolve modest limits when `rlimit` / `NEXUS_RLIMIT` is on.
#[must_use]
pub(crate) fn limits_from(env: &ShellEnvironment) -> Option<ChildLimits> {
    if !rlimit_enabled(env) {
        return None;
    }
    Some(ChildLimits {
        cpu_secs: override_u64(env, "rlimit_cpu", "NEXUS_RLIMIT_CPU", DEFAULT_CPU),
        nofile: override_u64(env, "rlimit_nofile", "NEXUS_RLIMIT_NOFILE", DEFAULT_NOFILE),
        #[cfg(target_os = "linux")]
        as_bytes: override_u64(env, "rlimit_as", "NEXUS_RLIMIT_AS", DEFAULT_AS),
        #[cfg(target_os = "linux")]
        nproc: override_u64(env, "rlimit_nproc", "NEXUS_RLIMIT_NPROC", DEFAULT_NPROC),
    })
}

/// Apply limits in the child (async-signal-safe `setrlimit` only).
pub(crate) fn apply(limits: &ChildLimits) -> io::Result<()> {
    set(Resource::RLIMIT_CPU, limits.cpu_secs)?;
    set(Resource::RLIMIT_NOFILE, limits.nofile)?;
    #[cfg(target_os = "linux")]
    set(Resource::RLIMIT_AS, limits.as_bytes)?;
    #[cfg(target_os = "linux")]
    set(Resource::RLIMIT_NPROC, limits.nproc)?;
    Ok(())
}

fn override_u64(env: &ShellEnvironment, local: &str, process: &str, default: u64) -> u64 {
    if let Some(v) = env.lookup(local) {
        if let Ok(n) = v.trim().parse() {
            return n;
        }
    }
    if let Ok(v) = std::env::var(process) {
        if let Ok(n) = v.trim().parse() {
            return n;
        }
    }
    default
}

fn set(resource: Resource, soft: u64) -> io::Result<()> {
    setrlimit(resource, soft, soft).map_err(|err| io::Error::from_raw_os_error(err as i32))
}
