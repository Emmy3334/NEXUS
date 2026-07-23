//! List pods in kubectl-style tables.

mod ready;

use super::age;
use super::client::try_client;
use super::table;
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;
use kube::ResourceExt;

use std::io;

/// Kubectl-like pod table lines (includes header).
pub fn list_pods_table(all_namespaces: bool) -> io::Result<Vec<String>> {
    let client = try_client()?;
    block_on(build_table(client, all_namespaces))?
}

/// Pod names in the default namespace (for Tab completion).
pub fn list_pod_names(prefix: &str) -> Vec<String> {
    let Ok(client) = try_client() else {
        return Vec::new();
    };
    match block_on(names_for(client, prefix)) {
        Ok(Ok(names)) => names,
        _ => Vec::new(),
    }
}

async fn names_for(client: kube::Client, prefix: &str) -> io::Result<Vec<String>> {
    let list = Api::<Pod>::default_namespaced(client)
        .list(&Default::default())
        .await
        .map_err(io_other)?;
    Ok(list
        .iter()
        .map(ResourceExt::name_any)
        .filter(|name| name.starts_with(prefix))
        .collect())
}

async fn build_table(client: kube::Client, all: bool) -> io::Result<Vec<String>> {
    let list = if all {
        Api::<Pod>::all(client)
            .list(&Default::default())
            .await
            .map_err(io_other)?
    } else {
        Api::<Pod>::default_namespaced(client)
            .list(&Default::default())
            .await
            .map_err(io_other)?
    };
    if list.items.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<Vec<String>> = list.iter().map(|pod| pod_row(pod, all)).collect();
    let headers: &[&str] = if all {
        &["NAMESPACE", "NAME", "READY", "STATUS", "RESTARTS", "AGE"]
    } else {
        &["NAME", "READY", "STATUS", "RESTARTS", "AGE"]
    };
    Ok(table::render(headers, &rows))
}

fn pod_row(pod: &Pod, with_ns: bool) -> Vec<String> {
    let mut row = Vec::new();
    if with_ns {
        row.push(pod.namespace().unwrap_or_else(|| "-".into()));
    }
    row.push(pod.name_any());
    row.push(ready::ready_count(pod));
    row.push(phase(pod));
    row.push(ready::restarts(pod).to_string());
    row.push(age::from_time(pod.metadata.creation_timestamp.as_ref()));
    row
}

fn phase(pod: &Pod) -> String {
    pod.status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".into())
}
