//! Native Kubernetes API helpers (`kube-rs`).

mod age;
mod client;
mod error;
mod logs;
mod nodes;
mod pods;
mod table;

pub use client::cluster_reachable;
pub use logs::pod_logs;
pub use nodes::list_nodes_table;
pub use pods::{list_pod_names, list_pods_table};

pub(crate) use error::format_err;
