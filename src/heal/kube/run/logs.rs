//! Fetch Pod logs into the shell writers.

use crate::tokio_rt::io_other;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, LogParams};
use kube::Client;

use std::io::{self, Write};

/// K8s log API returns a single combined stream (no portable stdout/stderr split).
/// Route by exit code: success → stdout, failure → stderr (closer to shell UX).
pub(super) async fn copy_logs(
    client: &Client,
    name: &str,
    code: u8,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> io::Result<()> {
    let pods: Api<Pod> = Api::default_namespaced(client.clone());
    let text = pods
        .logs(name, &LogParams::default())
        .await
        .map_err(io_other)?;
    if code == 0 {
        write!(stdout, "{text}")?;
    } else {
        write!(stderr, "{text}")?;
    }
    Ok(())
}
