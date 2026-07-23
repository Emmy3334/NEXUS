//! Fetch or follow pod logs.

mod follow;
mod snapshot;

pub use follow::pod_logs_follow;
pub use snapshot::pod_logs;
