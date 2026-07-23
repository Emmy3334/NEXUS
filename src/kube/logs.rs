//! Fetch recent logs from a pod in the default namespace.

use super::client::try_client;
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};

use std::io;

/// Return the last `tail` lines of logs for `pod` (default namespace).
pub fn pod_logs(pod: &str, tail: i64) -> io::Result<String> {
    let client = try_client()?;
    block_on(fetch(client, pod, tail))?
}

async fn fetch(client: kube::Client, pod: &str, tail: i64) -> io::Result<String> {
    let pods: Api<Pod> = Api::default_namespaced(client);
    let params = LogParams {
        tail_lines: Some(tail),
        ..Default::default()
    };
    pods.logs(pod, &params).await.map_err(io_other)
}
