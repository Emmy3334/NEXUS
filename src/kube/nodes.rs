//! List nodes in a kubectl-style table.

use super::age;
use super::client::try_client;
use super::table;
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Node;
use kube::api::Api;
use kube::ResourceExt;

use std::io;

/// Kubectl-like node table lines (includes header).
pub fn list_nodes_table() -> io::Result<Vec<String>> {
    let client = try_client()?;
    block_on(build_table(client))?
}

async fn build_table(client: kube::Client) -> io::Result<Vec<String>> {
    let nodes: Api<Node> = Api::all(client);
    let list = nodes.list(&Default::default()).await.map_err(io_other)?;
    if list.items.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<Vec<String>> = list.iter().map(node_row).collect();
    Ok(table::render(
        &["NAME", "STATUS", "ROLES", "AGE", "VERSION"],
        &rows,
    ))
}

fn node_row(node: &Node) -> Vec<String> {
    vec![
        node.name_any(),
        node_ready(node),
        node_roles(node),
        age::from_time(node.metadata.creation_timestamp.as_ref()),
        node_version(node),
    ]
}

fn node_ready(node: &Node) -> String {
    let ready = node
        .status
        .as_ref()
        .and_then(|s| s.conditions.as_ref())
        .into_iter()
        .flatten()
        .any(|c| c.type_ == "Ready" && c.status == "True");
    if ready {
        "Ready".into()
    } else {
        "NotReady".into()
    }
}

fn node_roles(node: &Node) -> String {
    let Some(labels) = &node.metadata.labels else {
        return "<none>".into();
    };
    let mut roles: Vec<&str> = labels
        .keys()
        .filter_map(|k| k.strip_prefix("node-role.kubernetes.io/"))
        .collect();
    if roles.is_empty() {
        return "<none>".into();
    }
    roles.sort_unstable();
    roles.join(",")
}

fn node_version(node: &Node) -> String {
    node.status
        .as_ref()
        .and_then(|s| s.node_info.as_ref())
        .map(|i| i.kubelet_version.clone())
        .unwrap_or_else(|| "<unknown>".into())
}
