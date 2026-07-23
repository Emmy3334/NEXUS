//! List pods in kubectl-style tables.

mod ready;

use super::age;
use super::client::try_client;
use super::scope::PodScope;
use super::table;
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;
use kube::ResourceExt;

use std::io;

pub(crate) use ready::{ready_count, restarts};

/// Kubectl-like pod table lines (includes header).
pub fn list_pods_table(scope: &PodScope) -> io::Result<Vec<String>> {
    let client = try_client()?;
    block_on(build_table(client, scope))?
}

/// Pod names for Tab completion (`namespace` `None` ⇒ default ns).
pub fn list_pod_names(prefix: &str, namespace: Option<&str>) -> Vec<String> {
    let Ok(client) = try_client() else {
        return Vec::new();
    };
    match block_on(names_for(client, prefix, namespace)) {
        Ok(Ok(names)) => names,
        _ => Vec::new(),
    }
}

async fn names_for(
    client: kube::Client,
    prefix: &str,
    namespace: Option<&str>,
) -> io::Result<Vec<String>> {
    let list = match namespace {
        Some(ns) => Api::<Pod>::namespaced(client, ns)
            .list(&Default::default())
            .await
            .map_err(io_other)?,
        None => Api::<Pod>::default_namespaced(client)
            .list(&Default::default())
            .await
            .map_err(io_other)?,
    };
    Ok(list
        .iter()
        .map(ResourceExt::name_any)
        .filter(|name| name.starts_with(prefix))
        .collect())
}

async fn build_table(client: kube::Client, scope: &PodScope) -> io::Result<Vec<String>> {
    let with_ns = scope.with_ns_column();
    let list = match scope {
        PodScope::All => Api::<Pod>::all(client)
            .list(&Default::default())
            .await
            .map_err(io_other)?,
        PodScope::Default => Api::<Pod>::default_namespaced(client)
            .list(&Default::default())
            .await
            .map_err(io_other)?,
        PodScope::Namespace(ns) => Api::<Pod>::namespaced(client, ns)
            .list(&Default::default())
            .await
            .map_err(io_other)?,
    };
    if list.items.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<Vec<String>> = list.iter().map(|pod| pod_row(pod, with_ns)).collect();
    let headers: &[&str] = if with_ns {
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
    row.push(ready_count(pod));
    row.push(phase(pod));
    row.push(restarts(pod).to_string());
    row.push(age::from_time(pod.metadata.creation_timestamp.as_ref()));
    row
}

fn phase(pod: &Pod) -> String {
    pod.status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".into())
}
