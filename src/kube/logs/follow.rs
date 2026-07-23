//! Follow pod logs (`Api::log_stream` + `LogParams.follow`).

use super::super::client::try_client;
use super::super::interrupt::{self, cancelled};
use crate::tokio_rt::{block_on, io_other};
use futures_util::{AsyncBufReadExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};
use std::io::{self, Write};
use std::time::Duration;

/// Stream pod logs to `out` until EOF or Ctrl-C.
pub fn pod_logs_follow(
    pod: &str,
    namespace: Option<&str>,
    tail: i64,
    out: &mut dyn Write,
) -> io::Result<()> {
    tracing::debug!(pod, ?namespace, tail, "kube pod_logs_follow");
    let client = try_client()?;
    interrupt::with_sigint_cancel(|| block_on(stream(client, pod, namespace, tail, out)))?
}

async fn stream(
    client: kube::Client,
    pod: &str,
    namespace: Option<&str>,
    tail: i64,
    out: &mut dyn Write,
) -> io::Result<()> {
    let pods = match namespace {
        Some(ns) => Api::<Pod>::namespaced(client, ns),
        None => Api::<Pod>::default_namespaced(client),
    };
    let params = LogParams {
        follow: true,
        tail_lines: Some(tail),
        ..Default::default()
    };
    let mut lines = pods
        .log_stream(pod, &params)
        .await
        .map_err(io_other)?
        .lines();
    loop {
        if cancelled() {
            return Ok(());
        }
        match tokio::time::timeout(Duration::from_millis(200), lines.try_next()).await {
            Ok(Ok(Some(line))) => {
                writeln!(out, "{line}")?;
                out.flush()?;
            }
            Ok(Ok(None)) => return Ok(()),
            Ok(Err(err)) => return Err(io_other(err)),
            Err(_) => {}
        }
    }
}
