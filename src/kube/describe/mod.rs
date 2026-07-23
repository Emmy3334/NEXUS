//! `describe`-style pod summary.

mod format;

use super::client::try_client;
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;

use std::io;

/// Multi-line describe text for one pod.
pub fn describe_pod(name: &str, namespace: Option<&str>) -> io::Result<String> {
    tracing::debug!(name, ?namespace, "kube describe_pod");
    let client = try_client()?;
    block_on(fetch(client, name, namespace))?
}

async fn fetch(client: kube::Client, name: &str, namespace: Option<&str>) -> io::Result<String> {
    let pods = match namespace {
        Some(ns) => Api::<Pod>::namespaced(client, ns),
        None => Api::<Pod>::default_namespaced(client),
    };
    let pod = pods.get(name).await.map_err(io_other)?;
    Ok(format::format_pod(&pod))
}
