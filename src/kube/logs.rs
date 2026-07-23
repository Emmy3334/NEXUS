//! Fetch recent logs from a pod.

use super::client::try_client;
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};

use std::io;

/// Return the last `tail` lines of logs for `pod`.
///
/// `namespace` `None` uses the kubeconfig default namespace.
pub fn pod_logs(pod: &str, namespace: Option<&str>, tail: i64) -> io::Result<String> {
    let client = try_client()?;
    block_on(fetch(client, pod, namespace, tail))?
}

async fn fetch(
    client: kube::Client,
    pod: &str,
    namespace: Option<&str>,
    tail: i64,
) -> io::Result<String> {
    let pods = match namespace {
        Some(ns) => Api::<Pod>::namespaced(client, ns),
        None => Api::<Pod>::default_namespaced(client),
    };
    let params = LogParams {
        tail_lines: Some(tail),
        ..Default::default()
    };
    pods.logs(pod, &params).await.map_err(io_other)
}
