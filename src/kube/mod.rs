//! Native Kubernetes API helpers (`kube-rs`).

mod age;
mod client;
mod describe;
mod error;
mod logs;
mod namespaces;
mod nodes;
mod pods;
mod scope;
mod table;

pub use client::cluster_reachable;
pub use describe::describe_pod;
pub use logs::pod_logs;
pub use namespaces::list_namespace_names;
pub use nodes::list_nodes_table;
pub use pods::{list_pod_names, list_pods_table};
pub use scope::PodScope;

pub(crate) use error::format_err;
