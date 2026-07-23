//! Thin non-TTY `@kube exec` via kube-rs websockets.

use super::client::try_client;
use super::interrupt::{self, cancelled};
use crate::tokio_rt::{block_on, io_other};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Status;
use kube::api::{Api, AttachParams, AttachedProcess};
use tokio::io::AsyncReadExt;

use std::io::{self, Write};
use std::time::Duration;

/// Run `command` in `pod` and copy stdout/stderr (no TTY / no stdin).
pub fn pod_exec(
    pod: &str,
    namespace: Option<&str>,
    command: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    tracing::debug!(pod, ?namespace, ?command, "kube pod_exec");
    if command.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "exec requires a command",
        ));
    }
    let client = try_client()?;
    interrupt::with_sigint_cancel(|| {
        block_on(run(client, pod, namespace, command, stdout, stderr))
    })?
}

async fn run(
    client: kube::Client,
    pod: &str,
    namespace: Option<&str>,
    command: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<u8> {
    let pods = match namespace {
        Some(ns) => Api::<Pod>::namespaced(client, ns),
        None => Api::<Pod>::default_namespaced(client),
    };
    let ap = AttachParams::default()
        .stdin(false)
        .stdout(true)
        .stderr(true);
    let mut attached = pods
        .exec(pod, command.iter().map(String::as_str), &ap)
        .await
        .map_err(io_other)?;
    let status_fut = attached.take_status();
    let (out_bytes, err_bytes) = drain_stdio(&mut attached).await?;
    stdout.write_all(&out_bytes)?;
    stderr.write_all(&err_bytes)?;
    let code = resolve_status(status_fut).await;
    let _ = attached.join().await;
    Ok(code)
}

async fn drain_stdio(attached: &mut AttachedProcess) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let mut out_r = attached.stdout();
    let mut err_r = attached.stderr();
    let out = async {
        let mut buf = Vec::new();
        if let Some(ref mut r) = out_r {
            read_cancel(r, &mut buf).await?;
        }
        Ok::<_, io::Error>(buf)
    };
    let err = async {
        let mut buf = Vec::new();
        if let Some(ref mut r) = err_r {
            read_cancel(r, &mut buf).await?;
        }
        Ok::<_, io::Error>(buf)
    };
    let (out, err) = tokio::join!(out, err);
    if cancelled() {
        attached.abort();
    }
    Ok((out?, err?))
}

async fn read_cancel(
    reader: &mut (impl AsyncReadExt + Unpin),
    out: &mut Vec<u8>,
) -> io::Result<()> {
    let mut buf = [0_u8; 4096];
    loop {
        if cancelled() {
            return Ok(());
        }
        match tokio::time::timeout(Duration::from_millis(200), reader.read(&mut buf)).await {
            Ok(Ok(0)) => return Ok(()),
            Ok(Ok(n)) => out.extend_from_slice(&buf[..n]),
            Ok(Err(err)) => return Err(err),
            Err(_) => {}
        }
    }
}

async fn resolve_status(
    status_fut: Option<impl std::future::Future<Output = Option<Status>>>,
) -> u8 {
    let Some(fut) = status_fut else {
        return 0;
    };
    match fut.await {
        Some(st) if st.status.as_deref() == Some("Success") => 0,
        Some(st) => exit_from_status(&st).unwrap_or(1),
        None => 0,
    }
}

fn exit_from_status(st: &Status) -> Option<u8> {
    st.details
        .as_ref()?
        .causes
        .as_ref()?
        .iter()
        .find_map(|c| c.message.as_deref()?.parse().ok())
}
